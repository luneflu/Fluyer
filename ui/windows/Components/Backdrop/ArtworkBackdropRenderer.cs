using System.Numerics;
using System.Runtime.InteropServices;
using Fluyer.Core.Rendering;
using Vortice.Direct3D;
using Vortice.Direct3D11;
using Vortice.D3DCompiler;
using Vortice.DXGI;
using Vortice.Mathematics;
using Windows.Graphics.Imaging;
using Windows.Storage.Streams;
using System.Runtime.InteropServices.WindowsRuntime;

namespace Fluyer.Components;

/// <summary>
/// D3D11 port of <c>ArtworkBackdropRenderer</c> from
/// <c>ui/macos/Sources/Components/Backdrop/ArtworkBackdropRenderer.swift</c>.
///
/// Same three-stage pipeline, same uniforms, same timing:
///
/// 1. Rotation — the current + previous artwork drawn as three fullscreen
///    instances spinning at 60/45/35s per revolution (painter-ordered, so the
///    viewport corners fall through to the slower layers).
/// 2. Blur — a 4x4 box downsample to quarter-res, then a separable gaussian
///    there at MPS's sigma. A true lowpass; the output is blurred to mush
///    either way, and at 240x135 even a ~77-tap kernel is trivially cheap.
/// 3. Pinch — the subdivided Music.app warp mesh, warped by the same
///    smoothstep phase, with the same saturation-2 + dark scrim + floor/ceiling.
///
/// Presentation differs from macOS by necessity: WinUI 3's SwapChainPanel does
/// not expose the UWP ISwapChainPanelNative contract (verified: QI returns
/// E_NOINTERFACE and the IID is absent from Microsoft.UI.Xaml.dll), so the
/// final frame is read back through a staging texture and uploaded to XAML as
/// a bitmap — the same display path as the static fallback. Deviations, all
/// visually negligible and documented: separable gaussian at quarter-res
/// instead of MPS gaussian at half-res; Clamp
/// instead of MPS zero edge mode (avoids a dark halo at the frame edge);
/// float instead of Metal half precision; intermediates capped at 960px
/// (macOS renders rotation/blur at full drawable size — indistinguishable
/// once blurred; the final frame stays at display resolution so the
/// output dither reaches the screen 1:1).
///
/// UI-thread-affine. Any failure throws <see cref="BackdropUnavailableException"/>;
/// per-frame errors never propagate — the caller stops the loop and shows the
/// static fallback instead (an ambient visual must never crash the app).
/// </summary>
public sealed class ArtworkBackdropRenderer : IDisposable
{
    public sealed class BackdropUnavailableException(string message, Exception? inner = null)
        : Exception(message, inner);

    private const int UniformBytes = 368;

    /// <summary>Long-edge cap for rotation and blur intermediates.</summary>
    private const int MaxEdge = 960;

    private static readonly Color4 ClearColor = new(0.07f, 0.07f, 0.09f, 1.0f);
    // BGRA memory order for the B8G8R8A8 fallback texture: (28, 28, 36) like
    // the macOS rgba8Unorm fallback pixel.
    private static readonly byte[] FallbackPixel = [36, 28, 28, 255];

    private readonly ID3D11Device _device;
    private readonly ID3D11DeviceContext _context;

    private readonly ID3D11VertexShader _rotationVs;
    private readonly ID3D11PixelShader _rotationPs;
    private readonly ID3D11VertexShader _fullscreenVs;
    private readonly ID3D11PixelShader _copyPs;
    private readonly ID3D11PixelShader _down4Ps;
    private readonly ID3D11PixelShader _gaussPs;
    private readonly ID3D11VertexShader _pinchVs;
    private readonly ID3D11PixelShader _pinchPs;
    private readonly ID3D11InputLayout _pinchLayout;
    private readonly ID3D11Buffer _meshVertices;
    private readonly ID3D11Buffer _meshIndices;
    private readonly int _meshIndexCount;
    private readonly ID3D11Buffer _uniforms;
    private readonly ID3D11Buffer _blurConstants;
    private readonly ID3D11RasterizerState _rasterizer;
    private readonly ID3D11SamplerState _sampler;

    private ID3D11Texture2D? _finalTex;
    private ID3D11RenderTargetView? _finalRtv;
    private ID3D11Texture2D? _stagingTex;
    private int _frameW;
    private int _frameH;
    private byte[] _frameBytes = [];

    private ID3D11Texture2D? _rotationTex;
    private ID3D11RenderTargetView? _rotationRtv;
    private ID3D11ShaderResourceView? _rotationSrv;
    private readonly ID3D11Texture2D?[] _blurTex = new ID3D11Texture2D?[2];
    private readonly ID3D11RenderTargetView?[] _blurRtv = new ID3D11RenderTargetView?[2];
    private readonly ID3D11ShaderResourceView?[] _blurSrv = new ID3D11ShaderResourceView?[2];
    private int _scratchW;
    private int _scratchH;
    private int _blurW;
    private int _blurH;
    private float _blurSigma = 1;

    private ID3D11Texture2D? _current;
    private ID3D11ShaderResourceView? _currentSrv;
    private ID3D11Texture2D? _previous;
    private ID3D11ShaderResourceView? _previousSrv;
    private ID3D11Texture2D? _pending;
    private ID3D11ShaderResourceView? _pendingSrv;
    private double? _transitionStart;

    private readonly long _epoch = DateTime.UtcNow.Ticks;
    private float _animationTime;
    private double? _lastFrameTime;
    private bool _disposed;

    public bool ReduceMotion { get; set; }

    public ArtworkBackdropRenderer()
    {
        try
        {
            _device = CreateDevice();
            _context = _device.ImmediateContext;
            _rotationVs = CompileVertex("rotation_vs");
            _rotationPs = CompilePixel("rotation_ps");
            _fullscreenVs = CompileVertex("fullscreen_vs");
            _copyPs = CompilePixel("copy_ps");
            _down4Ps = CompilePixel("down4_ps");
            _gaussPs = CompilePixel("gauss_ps");
            _pinchVs = CompileMeshVertex(out _pinchLayout!);
            _pinchPs = CompilePixel("pinch_ps");

            var (vertices, indices) = BackdropMesh.MakeMesh();
            _meshVertices = _device.CreateBuffer(vertices, BindFlags.VertexBuffer);
            _meshIndices = _device.CreateBuffer(indices, BindFlags.IndexBuffer);
            _meshIndexCount = indices.Length;

            _uniforms = _device.CreateBuffer(new BufferDescription(
                UniformBytes, BindFlags.ConstantBuffer, ResourceUsage.Dynamic,
                CpuAccessFlags.Write, ResourceOptionFlags.None, 0));
            _blurConstants = _device.CreateBuffer(new BufferDescription(
                16, BindFlags.ConstantBuffer, ResourceUsage.Dynamic,
                CpuAccessFlags.Write, ResourceOptionFlags.None, 0));
            _rasterizer = _device.CreateRasterizerState(new RasterizerDescription(CullMode.None, FillMode.Solid));
            _sampler = _device.CreateSamplerState(new SamplerDescription(
                Filter.MinMagMipLinear, TextureAddressMode.Clamp,
                TextureAddressMode.Clamp, TextureAddressMode.Clamp));

            var fallback = CreateArtworkTexture(1, 1, FallbackPixel);
            _current = fallback.Texture;
            _currentSrv = fallback.View;
        }
        catch (Exception ex) when (ex is not BackdropUnavailableException)
        {
            Dispose();
            throw new BackdropUnavailableException("Direct3D backdrop unavailable.", ex);
        }
    }

    // MARK: - Public driver (mirrors MTKViewDelegate + setImage)

    /// <summary>
    /// Upload new artwork (JPEG bytes, any size); null restores the fallback
    /// color. Decode is awaited, never blocked on, so the UI thread stays free.
    /// </summary>
    public async Task SetImageAsync(byte[]? jpeg)
    {
        ThrowIfDisposed();
        ID3D11Texture2D texture;
        ID3D11ShaderResourceView view;
        if (jpeg is null || jpeg.Length == 0)
        {
            (texture, view) = CreateArtworkTexture(1, 1, FallbackPixel);
        }
        else
        {
            var (pixels, w, h) = await DecodeBgraAsync(jpeg).ConfigureAwait(true);
            (texture, view) = CreateArtworkTexture(w, h, pixels);
        }
        _pending?.Dispose();
        _pendingSrv?.Dispose();
        _pending = texture;
        _pendingSrv = view;
    }

    /// <summary>
    /// Render one frame for a <paramref name="targetWidth"/> x
    /// <paramref name="targetHeight"/> view (physical pixels). The returned
    /// buffer is BGRA, <c>frameWidth * frameHeight * 4</c> bytes, and stays
    /// valid until the next call. Returns false when there is nothing to draw.
    /// </summary>
    public bool TryRender(
        int targetWidth, int targetHeight,
        out byte[] pixels, out int frameWidth, out int frameHeight)
    {
        ThrowIfDisposed();
        pixels = [];
        frameWidth = frameHeight = 0;
        if (targetWidth <= 0 || targetHeight <= 0)
        {
            return false;
        }

        var clock = (DateTime.UtcNow.Ticks - _epoch) / (double)TimeSpan.TicksPerSecond;
        // Freeze while hidden; changing Reduce Motion never jumps animation phase.
        var delta = Math.Min(Math.Max(clock - (_lastFrameTime ?? clock), 0), 1.0 / 15);
        _animationTime += (float)delta / (ReduceMotion ? 10 : 1);
        _lastFrameTime = clock;

        EnsureFrame(targetWidth, targetHeight);
        EnsureScratch(targetWidth, targetHeight);
        if (_transitionStart.HasValue && clock - _transitionStart.Value >= 0.5)
        {
            _previous?.Dispose();
            _previousSrv?.Dispose();
            _previous = null;
            _previousSrv = null;
            _transitionStart = null;
        }
        if (!_transitionStart.HasValue && _pending is not null)
        {
            _previous?.Dispose();
            _previousSrv?.Dispose();
            _previous = _current;
            _previousSrv = _currentSrv;
            _current = _pending;
            _currentSrv = _pendingSrv;
            _pending = null;
            _pendingSrv = null;
            _transitionStart = clock;
        }

        var uniforms = BackdropUniforms.Create((float)targetWidth / targetHeight);
        uniforms.Time = _animationTime;
        uniforms.TextureTransitionMix = _transitionStart.HasValue
            ? Math.Max(0, 1 - (float)((clock - _transitionStart.Value) / 0.5))
            : 0;

        _context.RSSetState(_rasterizer);

        // Pass 1: three spinning artwork instances.
        uniforms.Saturation = 1.3f;
        UploadUniforms(uniforms);
        _context.OMSetRenderTargets(_rotationRtv!);
        _context.ClearRenderTargetView(_rotationRtv, ClearColor);
        _context.RSSetViewport(0, 0, _scratchW, _scratchH);
        _context.IASetPrimitiveTopology(PrimitiveTopology.TriangleList);
        _context.IASetInputLayout(null!);
        _context.VSSetShader(_rotationVs);
        _context.PSSetShader(_rotationPs);
        _context.VSSetConstantBuffer(0, _uniforms);
        _context.PSSetConstantBuffer(0, _uniforms);
        _context.PSSetSamplers(0, [_sampler]);
        _context.PSSetShaderResources(0, [_currentSrv!, _previousSrv ?? _currentSrv!]);
        _context.DrawInstanced(3, 3, 0, 0);
        UnbindShaderResources();

        // Pass 2-4: 4x4 box downsample to quarter-res, then a separable
        // gaussian there (horizontal, then vertical).
        UploadBlurConstants(new Vector4(1f / _scratchW, 1f / _scratchH, 0, 0));
        Blit(_down4Ps, _blurRtv[0]!, _rotationSrv!, _blurW, _blurH);
        UploadBlurConstants(new Vector4(1f / _blurW, 0, _blurSigma, 0));
        Blit(_gaussPs, _blurRtv[1]!, _blurSrv[0]!, _blurW, _blurH);
        UploadBlurConstants(new Vector4(0, 1f / _blurH, _blurSigma, 0));
        Blit(_gaussPs, _blurRtv[0]!, _blurSrv[1]!, _blurW, _blurH);
        var blurredSrv = _blurSrv[0]!;

        // Pass 8: warp mesh onto the final frame target.
        uniforms.Saturation = 2;
        UploadUniforms(uniforms);
        _context.OMSetRenderTargets(_finalRtv!);
        _context.ClearRenderTargetView(_finalRtv, ClearColor);
        _context.RSSetViewport(0, 0, _frameW, _frameH);
        _context.IASetInputLayout(_pinchLayout);
        _context.IASetVertexBuffer(0, _meshVertices, 56);
        _context.IASetIndexBuffer(_meshIndices, Format.R32_UInt, 0);
        _context.VSSetShader(_pinchVs);
        _context.PSSetShader(_pinchPs);
        _context.VSSetConstantBuffer(0, _uniforms);
        _context.PSSetConstantBuffer(0, _uniforms);
        _context.PSSetSamplers(0, [_sampler]);
        _context.PSSetShaderResources(0, [blurredSrv]);
        _context.DrawIndexed((uint)_meshIndexCount, 0, 0);
        UnbindShaderResources();

        // Present via readback: the frame is mush, so a staging copy presents
        // identically to a swapchain with none of the interop risk.
        _context.CopyResource(_stagingTex!, _finalTex!);
        var box = _context.Map(_stagingTex!, 0, MapMode.Read, Vortice.Direct3D11.MapFlags.None);
        try
        {
            for (var y = 0; y < _frameH; y++)
            {
                Marshal.Copy(box.DataPointer + y * (int)box.RowPitch, _frameBytes, y * _frameW * 4, _frameW * 4);
            }
        }
        finally
        {
            _context.Unmap(_stagingTex, 0);
        }

        pixels = _frameBytes;
        frameWidth = _frameW;
        frameHeight = _frameH;
        return true;
    }

    // MARK: - Setup

    private static ID3D11Device CreateDevice()
    {
        var levels = new[] { FeatureLevel.Level_11_1, FeatureLevel.Level_11_0 };
        Exception? hardwareError = null;
        try
        {
            return D3D11.D3D11CreateDevice(
                DriverType.Hardware, DeviceCreationFlags.BgraSupport, levels);
        }
        catch (Exception ex)
        {
            hardwareError = ex;
        }
        try
        {
            return D3D11.D3D11CreateDevice(
                DriverType.Warp, DeviceCreationFlags.BgraSupport, levels);
        }
        catch (Exception ex)
        {
            throw new BackdropUnavailableException(
                $"D3D11 device creation failed (hardware: {hardwareError?.Message}; warp: {ex.Message}).");
        }
    }

    private ID3D11VertexShader CompileVertex(string entry)
    {
        var bytecode = Compiler.Compile(
            BackdropShaders.Source, entry, "backdrop.hlsl", "vs_5_0",
            ShaderFlags.None, EffectFlags.None).ToArray();
        return _device.CreateVertexShader(bytecode, null);
    }

    private ID3D11VertexShader CompileMeshVertex(out ID3D11InputLayout layout)
    {
        var bytecode = Compiler.Compile(
            BackdropShaders.Source, "pinch_vs", "backdrop.hlsl", "vs_5_0",
            ShaderFlags.None, EffectFlags.None).ToArray();
        var shader = _device.CreateVertexShader(bytecode, null);
        layout = _device.CreateInputLayout(
        [
            new InputElementDescription("POSITION", 0, Format.R32G32B32A32_Float, 0, 0),
            new InputElementDescription("FROMPOS", 0, Format.R32G32B32A32_Float, 16, 0),
            new InputElementDescription("TOPOS", 0, Format.R32G32B32A32_Float, 32, 0),
            new InputElementDescription("TEXCOORD", 0, Format.R32G32_Float, 48, 0),
        ],
            bytecode);
        return shader;
    }

    private ID3D11PixelShader CompilePixel(string entry)
    {
        var bytecode = Compiler.Compile(
            BackdropShaders.Source, entry, "backdrop.hlsl", "ps_5_0",
            ShaderFlags.None, EffectFlags.None).ToArray();
        return _device.CreatePixelShader(bytecode, null);
    }

    // MARK: - Per-frame helpers

    private void EnsureFrame(int targetW, int targetH)
    {
        // Dither at display resolution: upscaling a smaller frame smooths it away.
        var w = targetW;
        var h = targetH;
        if (w == _frameW && h == _frameH && _finalTex is not null)
        {
            return;
        }
        foreach (var d in new IDisposable?[] { _finalTex, _finalRtv, _stagingTex })
        {
            d?.Dispose();
        }
        _finalTex = _device.CreateTexture2D(new Texture2DDescription
        {
            Width = (uint)w,
            Height = (uint)h,
            MipLevels = 1,
            ArraySize = 1,
            Format = Format.B8G8R8A8_UNorm,
            SampleDescription = new SampleDescription(1, 0),
            Usage = ResourceUsage.Default,
            BindFlags = BindFlags.RenderTarget | BindFlags.ShaderResource,
            CPUAccessFlags = CpuAccessFlags.None,
            MiscFlags = ResourceOptionFlags.None,
        });
        _finalRtv = _device.CreateRenderTargetView(_finalTex, null);
        _stagingTex = _device.CreateTexture2D(new Texture2DDescription
        {
            Width = (uint)w,
            Height = (uint)h,
            MipLevels = 1,
            ArraySize = 1,
            Format = Format.B8G8R8A8_UNorm,
            SampleDescription = new SampleDescription(1, 0),
            Usage = ResourceUsage.Staging,
            BindFlags = BindFlags.None,
            CPUAccessFlags = CpuAccessFlags.Read,
            MiscFlags = ResourceOptionFlags.None,
        });
        _frameW = w;
        _frameH = h;
        _frameBytes = new byte[w * h * 4];
    }

    private void EnsureScratch(int targetW, int targetH)
    {
        // ponytail: cap intermediate long edge at 960px, like Metal resize().
        var scale = Math.Min(1.0, MaxEdge / (double)Math.Max(targetW, targetH));
        var w = Math.Max(1, (int)(targetW * scale));
        var h = Math.Max(1, (int)(targetH * scale));
        if (w == _scratchW && h == _scratchH && _rotationTex is not null)
        {
            return;
        }
        foreach (var d in new IDisposable?[] {
            _rotationTex, _rotationRtv, _rotationSrv,
            _blurTex[0], _blurRtv[0], _blurSrv[0],
            _blurTex[1], _blurRtv[1], _blurSrv[1] })
        {
            d?.Dispose();
        }
        (_rotationTex, _rotationRtv, _rotationSrv) = CreateScratch(w, h);
        _blurW = Math.Max(1, w / 4);
        _blurH = Math.Max(1, h / 4);
        for (var i = 0; i < 2; i++)
        {
            (_blurTex[i], _blurRtv[i], _blurSrv[i]) = CreateScratch(_blurW, _blurH);
        }
        _scratchW = w;
        _scratchH = h;

        // MPS sigma, ported verbatim: floor(hypot(W,H) * 0.045394707) * scale,
        // then scaled into quarter-res blur texels.
        var sigmaScratch = Math.Floor(Math.Sqrt((double)targetW * targetW + (double)targetH * targetH) * 0.045394707) * scale;
        _blurSigma = (float)Math.Max(sigmaScratch * _blurW / _scratchW, 0.5);
    }

    private (ID3D11Texture2D Tex, ID3D11RenderTargetView Rtv, ID3D11ShaderResourceView Srv) CreateScratch(int w, int h)
    {
        var tex = _device.CreateTexture2D(new Texture2DDescription
        {
            Width = (uint)w,
            Height = (uint)h,
            MipLevels = 1,
            ArraySize = 1,
            // Preserve precision through rotation and every blur iteration;
            // quantize once at the final BGRA8 presentation target.
            Format = Format.R16G16B16A16_Float,
            SampleDescription = new SampleDescription(1, 0),
            Usage = ResourceUsage.Default,
            BindFlags = BindFlags.RenderTarget | BindFlags.ShaderResource,
            CPUAccessFlags = CpuAccessFlags.None,
            MiscFlags = ResourceOptionFlags.None,
        });
        return (tex, _device.CreateRenderTargetView(tex, null), _device.CreateShaderResourceView(tex, null));
    }

    private void UploadUniforms(in BackdropUniforms uniforms)
    {
        var box = _context.Map(_uniforms, MapMode.WriteDiscard, Vortice.Direct3D11.MapFlags.None);
        try
        {
            Marshal.StructureToPtr(uniforms, box.DataPointer, false);
        }
        finally
        {
            _context.Unmap(_uniforms, 0);
        }
    }

    private void UploadBlurConstants(Vector4 c)
    {
        var box = _context.Map(_blurConstants, MapMode.WriteDiscard, Vortice.Direct3D11.MapFlags.None);
        try
        {
            Marshal.StructureToPtr(c, box.DataPointer, false);
        }
        finally
        {
            _context.Unmap(_blurConstants, 0);
        }
    }

    /// <summary>Fullscreen-triangle blit: no vertex buffer, source on t0.</summary>
    private void Blit(ID3D11PixelShader ps, ID3D11RenderTargetView rtv, ID3D11ShaderResourceView srv, int w, int h)
    {
        _context.OMSetRenderTargets(rtv);
        _context.RSSetViewport(0, 0, w, h);
        _context.IASetPrimitiveTopology(PrimitiveTopology.TriangleList);
        _context.IASetInputLayout(null!);
        _context.VSSetShader(_fullscreenVs);
        _context.PSSetShader(ps);
        _context.PSSetConstantBuffer(1, _blurConstants);
        _context.PSSetSamplers(0, [_sampler]);
        _context.PSSetShaderResources(0, [srv]);
        _context.Draw(3, 0);
        UnbindShaderResources();
    }

    private void UnbindShaderResources()
        => _context.PSSetShaderResources(0, [null!, null!]);

    private (ID3D11Texture2D Texture, ID3D11ShaderResourceView View) CreateArtworkTexture(int w, int h, byte[] bgra)
    {
        var handle = GCHandle.Alloc(bgra, GCHandleType.Pinned);
        try
        {
            var tex = _device.CreateTexture2D(new Texture2DDescription
            {
                Width = (uint)w,
                Height = (uint)h,
                MipLevels = 1,
                ArraySize = 1,
                Format = Format.B8G8R8A8_UNorm,
                SampleDescription = new SampleDescription(1, 0),
                Usage = ResourceUsage.Immutable,
                BindFlags = BindFlags.ShaderResource,
                CPUAccessFlags = CpuAccessFlags.None,
                MiscFlags = ResourceOptionFlags.None,
            }, new SubresourceData(handle.AddrOfPinnedObject(), (uint)(w * 4), 0));
            return (tex, _device.CreateShaderResourceView(tex, null));
        }
        finally
        {
            handle.Free();
        }
    }

    private static async Task<(byte[] Pixels, int Width, int Height)> DecodeBgraAsync(byte[] jpeg)
    {
        using var stream = new InMemoryRandomAccessStream();
        await stream.WriteAsync(jpeg.AsBuffer()).AsTask().ConfigureAwait(false);
        stream.Seek(0);
        var decoder = await BitmapDecoder.CreateAsync(stream).AsTask().ConfigureAwait(false);
        // Backdrop is blurred to mush, so 1200px is ample (macOS backdropPixels).
        const uint MaxEdge = 1200;
        var w = (int)decoder.PixelWidth;
        var h = (int)decoder.PixelHeight;
        var transform = new BitmapTransform();
        var scale = MaxEdge / (float)Math.Max(w, h);
        if (scale < 1)
        {
            w = (int)(w * scale);
            h = (int)(h * scale);
            transform.ScaledWidth = (uint)w;
            transform.ScaledHeight = (uint)h;
        }
        var pixels = await decoder.GetPixelDataAsync(
            BitmapPixelFormat.Bgra8, BitmapAlphaMode.Ignore,
            transform, ExifOrientationMode.RespectExifOrientation,
            ColorManagementMode.DoNotColorManage).AsTask().ConfigureAwait(false);
        return (pixels.DetachPixelData(), w, h);
    }

    private void ThrowIfDisposed()
    {
        if (_disposed)
        {
            throw new ObjectDisposedException(nameof(ArtworkBackdropRenderer));
        }
    }

    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }
        _disposed = true;
        foreach (var d in new IDisposable?[] {
            _rasterizer, _sampler, _uniforms, _blurConstants,
            _meshVertices, _meshIndices, _pinchLayout,
            _rotationVs, _rotationPs, _fullscreenVs, _copyPs, _down4Ps, _gaussPs, _pinchVs, _pinchPs,
            _rotationTex, _rotationRtv, _rotationSrv,
            _blurTex[0], _blurRtv[0], _blurSrv[0],
            _blurTex[1], _blurRtv[1], _blurSrv[1],
            _current, _currentSrv, _previous, _previousSrv, _pending, _pendingSrv,
            _finalTex, _finalRtv, _stagingTex,
            _context, _device })
        {
            try
            {
                d?.Dispose();
            }
            catch
            {
            }
        }
    }
}
