namespace Fluyer.Rendering;

/// <summary>
/// Direct HLSL port of the Metal <c>shaderSource</c> in
/// <c>ui/macos/Sources/Rendering/ArtworkBackdropRenderer.swift</c>,
/// plus two helpers Metal gets for free:
///
/// - <c>copy_vs/copy_ps</c>: bilinear downsample of the rotation target
///   into the half-res blur chain.
/// - <c>kawase_vs/kawase_ps</c>: 4-tap Kawase blur, iterated. A single
///   gaussian kernel at MPS's sigma (~50px at 960p) would need hundreds of
///   taps; Kawase approximates the same mush in 5 cheap iterations.
///
/// Layout: the b0 cbuffer must stay byte-identical to
/// <see cref="Core.Rendering.BackdropUniforms"/> (368 bytes). HLSL vectors
/// cannot straddle 16-byte rows, so the 24 bytes of padding are six scalar
/// floats — float4+float2 would start at 112 and shift models[] off the CPU
/// layout (verified against fxc disassembly during development).
///
/// Orientation: D3D11 NDC maps (-1,-1) to the bottom-left, same as Metal,
/// and the fullscreen triangle below reproduces Metal's quad UV mapping
/// exactly (uv = (x/2+.5, -y/2+.5)). Pixels the triangle covers outside the
/// quad are discarded via the <c>ndc</c> varying, so the three spinning
/// layers composite exactly like Metal's painter-ordered instances.
/// </summary>
public static class BackdropShaders
{
    public const string Source = """
        cbuffer BackdropUniforms : register(b0) {
            float4x4 viewMatrix;
            float time;
            float textureTransitionMix;
            float meshWarpTimeScale;
            float saturation;
            float whiteScrimAlpha;
            float blackScrimAlpha;
            float factorForDarkMode;
            float factorForLightMode;
            float floorLevel;
            float ceilingLevel;
            // Six scalar floats, NOT float4+float2: HLSL vectors cannot
            // straddle 16-byte rows, so vector padding would start at 112
            // and shift models[] by 16 bytes off the 368-byte CPU layout.
            // Scalars pack tightly: 104..128, models[0] at 128.
            float pad0;
            float pad1;
            float pad2;
            float pad3;
            float pad4;
            float pad5;
            struct BackdropModel {
                float4x4 mtx;
                float timeScale;
                float3 modelPad;
            };
            BackdropModel models[3];
        };

        cbuffer BlurConstants : register(b1) {
            float2 blurTexelSize;
            float2 kawaseOffsetUv;
        };

        // One pair shared by every pass: the rotation pass reads the two
        // artwork generations, all later passes read the blur chain head.
        Texture2D srcA : register(t0);
        Texture2D srcB : register(t1);
        SamplerState linearClamp : register(s0);

        // NOTE on layout: the uploaded matrices (diagonal aspect scale,
        // translations) are transpose-invariant for column-vector application,
        // so mul(M, v) here equals Metal's M * v without a transpose step.

        float4x4 rotationOf(float angle) {
            float c = cos(angle);
            float s = sin(angle);
            float4x4 r;
            r[0] = float4(c, -s, 0, 0);
            r[1] = float4(s, c, 0, 0);
            r[2] = float4(0, 0, 1, 0);
            r[3] = float4(0, 0, 0, 1);
            return r;
        }

        float3 saturateColor(float3 rgb, float amount) {
            float luma = dot(rgb, float3(0.3, 0.59, 0.11));
            return luma + (rgb - luma) * amount;
        }

        struct RotationVsOut {
            float4 pos : SV_Position;
            float2 uv : TEXCOORD0;
            float2 ndc : TEXCOORD1;
        };

        RotationVsOut rotation_vs(uint vid : SV_VertexID, uint inst : SV_InstanceID) {
            float2 p = vid == 0 ? float2(-1, -1) : (vid == 1 ? float2(3, -1) : float2(-1, 3));
            float2 uv = float2(p.x * 0.5 + 0.5, -p.y * 0.5 + 0.5);
            float angle = time * 6.28318530718 / models[inst].timeScale;
            float4x4 r = rotationOf(angle);
            float4 v = float4(p, 0, 1);
            RotationVsOut o;
            o.pos = mul(viewMatrix, mul(r, mul(models[inst].mtx, mul(r, v))));
            o.uv = uv;
            o.ndc = p;
            return o;
        }

        float4 rotation_ps(RotationVsOut i) : SV_Target {
            if (any(abs(i.ndc) > float2(1, 1))) discard;
            float3 cur = srcA.Sample(linearClamp, i.uv).rgb;
            float3 prv = srcB.Sample(linearClamp, i.uv).rgb;
            float3 rgb = lerp(cur, prv, textureTransitionMix);
            return float4(saturateColor(rgb, saturation), 1);
        }

        struct FullscreenVsOut {
            float4 pos : SV_Position;
            float2 uv : TEXCOORD0;
        };

        FullscreenVsOut fullscreen_vs(uint vid : SV_VertexID) {
            float2 p = vid == 0 ? float2(-1, -1) : (vid == 1 ? float2(3, -1) : float2(-1, 3));
            FullscreenVsOut o;
            o.pos = float4(p, 0, 1);
            o.uv = float2(p.x * 0.5 + 0.5, -p.y * 0.5 + 0.5);
            return o;
        }

        float4 copy_ps(FullscreenVsOut i) : SV_Target {
            return srcA.Sample(linearClamp, i.uv);
        }

        float4 kawase_ps(FullscreenVsOut i) : SV_Target {
            float2 d = kawaseOffsetUv;
            float3 c0 = srcA.Sample(linearClamp, i.uv + float2(d.x, d.y)).rgb;
            float3 c1 = srcA.Sample(linearClamp, i.uv - float2(d.x, d.y)).rgb;
            float3 c2 = srcA.Sample(linearClamp, i.uv + float2(-d.x, d.y)).rgb;
            float3 c3 = srcA.Sample(linearClamp, i.uv + float2(d.x, -d.y)).rgb;
            return float4((c0 + c1 + c2 + c3) * 0.25, 1);
        }

        struct PinchVin {
            float4 position : POSITION;
            float4 fromPosition : FROMPOS;
            float4 toPosition : TOPOS;
            float2 uv : TEXCOORD;
        };

        struct PinchVsOut {
            float4 pos : SV_Position;
            float2 uv : TEXCOORD0;
        };

        PinchVsOut pinch_vs(PinchVin v) {
            float phase = acos(sin(time * 3.14159265359 / meshWarpTimeScale)) / 3.14159265359;
            float blend = phase * phase * (3 - 2 * phase);
            PinchVsOut o;
            o.pos = float4(lerp(v.fromPosition.xy, v.toPosition.xy, blend), 0, 1);
            o.uv = (v.uv - 0.5) * 0.8 + 0.5;
            return o;
        }

        float DitherNoise(uint2 p, uint frame, uint channel) {
            uint n = p.x * 1664525u + p.y * 1013904223u;
            n += frame * 747796405u + channel * 2891336453u;
            n ^= n >> 16;
            n *= 2246822519u;
            n ^= n >> 13;
            return (float)(n & 0xffffu) / 65535.0 * 2.0 - 1.0;
        }

        float4 pinch_ps(PinchVsOut i) : SV_Target {
            float3 rgb = min(saturateColor(srcA.Sample(linearClamp, i.uv).rgb, saturation), 0.995);
            if (factorForDarkMode > 0) {
                rgb = lerp(rgb, float3(0, 0, 0), blackScrimAlpha) - factorForDarkMode * 0.1;
            } else if (factorForLightMode > 0) {
                rgb = min(lerp(rgb, float3(1, 1, 1), whiteScrimAlpha) + factorForLightMode * 0.05, 1);
            }
            // Per-pixel temporal dither, ±1 LSB: final BGRA8 conversion would
            // otherwise show contour steps in smooth dark gradients. Hashing
            // pixel + frame avoids Bayer crosshatch, which VA/FRC processing
            // can preserve instead of averaging. Separate channel seeds stop
            // one shared error from becoming visible luminance contouring.
            uint2 px = (uint2)i.pos.xy;
            uint f = (uint)(time * 60.0);
            rgb = clamp(rgb, floorLevel, ceilingLevel);
            rgb.r += DitherNoise(px, f, 0) / 255.0;
            rgb.g += DitherNoise(px, f, 1) / 255.0;
            rgb.b += DitherNoise(px, f, 2) / 255.0;
            return float4(saturate(rgb), 1);
        }

        """;
}
