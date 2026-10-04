using Fluyer.Core.State;
using Fluyer.Rendering;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.Graphics.Imaging;
using Windows.Storage.Streams;
using System.Runtime.InteropServices.WindowsRuntime;

namespace Fluyer.Views;

/// <summary>
/// Animated, blurred artwork backdrop behind the whole window. Refresh key
/// mirrors macOS (<c>track path + palette</c>): artwork re-uploads only when
/// the song or its colors change; the GPU animates continuously at 15fps and
/// each frame is uploaded as a bitmap (capped ~640px long edge — blurred to
/// mush either way). When Direct3D is unavailable, falls
/// back to the core's pre-blurred ambient frame as a static bitmap.
/// </summary>
public sealed partial class AnimatedBackgroundView : UserControl
{
    public static readonly DependencyProperty StateProperty =
        DependencyProperty.Register(nameof(State), typeof(AppState), typeof(AnimatedBackgroundView),
            new PropertyMetadata(null, OnStateChanged));

    public AppState? State
    {
        get => (AppState?)GetValue(StateProperty);
        set => SetValue(StateProperty, value);
    }

    private ArtworkBackdropRenderer? _renderer;
    private SoftwareBitmapSource? _frameSource;
    private Microsoft.UI.Dispatching.DispatcherQueueTimer? _frameTimer;
    private string _appliedKey = string.Empty;
    private bool _frameBusy;
    private int _lastW;
    private int _lastH;

    public AnimatedBackgroundView()
    {
        InitializeComponent();
        Loaded += OnLoaded;
        Unloaded += OnUnloaded;
    }

    private static void OnStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (AnimatedBackgroundView)d;
        if (e.OldValue is AppState oldState)
        {
            oldState.Playback.PropertyChanged -= self.OnPlaybackChanged;
        }
        if (e.NewValue is AppState newState)
        {
            newState.Playback.PropertyChanged += self.OnPlaybackChanged;
        }
        _ = self.RefreshArtworkAsync();
    }

    private void OnPlaybackChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(PlaybackState.PlayView))
        {
            _ = RefreshArtworkAsync();
        }
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        try
        {
            _renderer = new ArtworkBackdropRenderer();
            _renderer.ReduceMotion = !AnimationsEnabled();
        }
        catch (ArtworkBackdropRenderer.BackdropUnavailableException ex)
        {
            System.Diagnostics.Debug.WriteLine($"Backdrop GPU unavailable, using static fallback: {ex.Message}");
            _renderer = null;
        }
        _frameSource = new SoftwareBitmapSource();
        Backdrop.Source = _frameSource;
        _ = RefreshArtworkAsync();
        StartFrameLoop();
    }

    private void OnUnloaded(object sender, RoutedEventArgs e)
    {
        StopFrameLoop();
        _renderer?.Dispose();
        _renderer = null;
    }

    private static bool AnimationsEnabled()
    {
        try
        {
            return new Windows.UI.ViewManagement.UISettings().AnimationsEnabled;
        }
        catch
        {
            return true;
        }
    }

    private void StartFrameLoop()
    {
        StopFrameLoop();
        if (_renderer is null)
        {
            return;
        }
        var queue = DispatcherQueue;
        _frameTimer = queue.CreateTimer();
        // Blurred to mush — 15fps at a capped size uploads ~1/4 the pixels
        // of 30fps full-res with no visible difference. List scrolls and
        // cover decodes then stop starving the frame loop (the starvation
        // reads as flicker).
        _frameTimer.Interval = TimeSpan.FromMilliseconds(1000.0 / 15);
        _frameTimer.Tick += (_, _) => TickFrame();
        _frameTimer.Start();
    }

    private void StopFrameLoop()
    {
        if (_frameTimer is not null)
        {
            _frameTimer.Stop();
            _frameTimer = null;
        }
    }

    private async void TickFrame()
    {
        if (_renderer is null || _frameSource is null || Visibility != Visibility.Visible)
        {
            return;
        }
        if (ActualWidth <= 0 || ActualHeight <= 0)
        {
            return;
        }
        if (_frameBusy)
        {
            return; // previous frame still uploading — skip, don't pile up
        }
        _frameBusy = true;
        try
        {
            // Cap: final frame dithers display detail away, so rendering past
            // ~640px long edge only costs D3D time and upload bytes. Ignore
            // sub-pixel layout churn too — reallocating D3D targets mid-loop
            // drops a frame (visible blink).
            const double MaxEdge = 640;
            var rawW = ActualWidth * CompositionScaleX();
            var rawH = ActualHeight * CompositionScaleY();
            var scale = Math.Min(1.0, MaxEdge / Math.Max(rawW, rawH));
            var targetW = Math.Max(1, (int)(rawW * scale));
            var targetH = Math.Max(1, (int)(rawH * scale));
            if (Math.Abs(targetW - _lastW) < 2 && Math.Abs(targetH - _lastH) < 2 && _lastW > 0)
            {
                targetW = _lastW;
                targetH = _lastH;
            }
            if (!_renderer.TryRender(targetW, targetH, out var pixels, out var w, out var h))
            {
                return;
            }
            _lastW = targetW;
            _lastH = targetH;
            var bitmap = new SoftwareBitmap(BitmapPixelFormat.Bgra8, w, h, BitmapAlphaMode.Premultiplied);
            bitmap.CopyFromBuffer(pixels.AsBuffer());
            await _frameSource.SetBitmapAsync(bitmap).AsTask().ConfigureAwait(true);
        }
        catch (Exception ex)
        {
            // Ambient visual only — stop the loop; the last good frame stays up.
            System.Diagnostics.Debug.WriteLine($"Backdrop frame failed: {ex.Message}");
            StopFrameLoop();
        }
        finally
        {
            _frameBusy = false;
        }
    }

    private float CompositionScaleX()
    {
        try
        {
            var root = XamlRoot;
            return root is null ? 1.0f : (float)root.RasterizationScale;
        }
        catch
        {
            return 1.0f;
        }
    }

    private float CompositionScaleY() => CompositionScaleX();

    private async Task RefreshArtworkAsync()
    {
        var state = State;
        var engine = state?.Engine;
        var track = state?.Playback.PlayView.Track;
        if (engine is null || track is null)
        {
            return;
        }
        var palette = string.Join(";", state!.Playback.PlayView.Palette.Select(c => $"{c.R},{c.G},{c.B}"));
        var key = $"{track.Path}|{palette}";
        if (key == _appliedKey)
        {
            return;
        }
        _appliedKey = key;

        // Backdrop source is blurred to mush, so 1200px is ample (macOS backdropPixels).
        var jpeg = await Task.Run(() => engine.GetCurrentThumbnail(1200)).ConfigureAwait(true);
        if (key != CurrentKey())
        {
            return; // track changed mid-load — drop the stale upload
        }
        if (_renderer is not null)
        {
            try
            {
                await _renderer.SetImageAsync(jpeg).ConfigureAwait(true);
                return;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"Backdrop upload failed, falling back: {ex.Message}");
                StopFrameLoop();
                _renderer?.Dispose();
                _renderer = null;
            }
        }
        await ShowStaticFallbackAsync(jpeg).ConfigureAwait(true);
    }

    private string CurrentKey()
    {
        var state = State;
        if (state is null)
        {
            return string.Empty;
        }
        var palette = string.Join(";", state.Playback.PlayView.Palette.Select(c => $"{c.R},{c.G},{c.B}"));
        return $"{state.Playback.PlayView.Track?.Path}|{palette}";
    }

    private async Task ShowStaticFallbackAsync(byte[]? jpeg)
    {
        if (_frameSource is null)
        {
            return;
        }
        if (jpeg is null || jpeg.Length == 0)
        {
            return;
        }
        try
        {
            var image = new BitmapImage();
            using var stream = new InMemoryRandomAccessStream();
            await stream.WriteAsync(jpeg.AsBuffer()).AsTask().ConfigureAwait(true);
            stream.Seek(0);
            await image.SetSourceAsync(stream).AsTask().ConfigureAwait(true);
            Backdrop.Source = image;
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Backdrop fallback decode failed: {ex.Message}");
        }
    }
}
