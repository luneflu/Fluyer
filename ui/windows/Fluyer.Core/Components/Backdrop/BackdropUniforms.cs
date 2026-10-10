using System.Numerics;
using System.Runtime.InteropServices;

namespace Fluyer.Core.Rendering;

/// <summary>
/// One Music.app-style model slot: transform plus spin speed.
/// Must stay 80 bytes (64 + 4 + 12), matching both the Swift
/// <c>BackdropModel</c> and the HLSL <c>BackdropModel</c>.
/// </summary>
[StructLayout(LayoutKind.Sequential)]
public struct BackdropModel
{
    public Matrix4x4 Matrix;
    public float TimeScale;
    public float Pad0;
    public float Pad1;
    public float Pad2;

    public BackdropModel(Matrix4x4 matrix, float timeScale)
    {
        Matrix = matrix;
        TimeScale = timeScale;
        Pad0 = Pad1 = Pad2 = 0;
    }
}

/// <summary>
/// Exact CPU mirror of the Metal <c>BackdropUniforms</c> layout from
/// <c>ui/macos/Sources/Components/Backdrop/ArtworkBackdropRenderer.swift</c>:
/// 368 bytes total, models at byte offsets 128 / 208 / 288.
///
/// Upload note: the matrices we store (diagonal aspect scale, translations)
/// are transpose-invariant for column-vector application, so the raw
/// <see cref="Matrix4x4"/> bytes can be copied into the HLSL column-major
/// cbuffer as-is and <c>mul(M, v)</c> matches Metal's <c>M * v</c> exactly.
/// </summary>
[StructLayout(LayoutKind.Explicit, Size = 368)]
public struct BackdropUniforms
{
    [FieldOffset(0)] public Matrix4x4 ViewMatrix;          // 0x00..0x40
    [FieldOffset(64)] public float Time;                   // 0x40
    [FieldOffset(68)] public float TextureTransitionMix;   // 0x44: old artwork weight
    [FieldOffset(72)] public float MeshWarpTimeScale;       // default 1.75
    [FieldOffset(76)] public float Saturation;              // 1.3 rotation, 2 pinch
    [FieldOffset(80)] public float WhiteScrimAlpha;        // 0
    [FieldOffset(84)] public float BlackScrimAlpha;         // 0.25
    [FieldOffset(88)] public float FactorForDarkMode;      // 0.2
    [FieldOffset(92)] public float FactorForLightMode;     // 0
    [FieldOffset(96)] public float Floor;                  // 0.07
    [FieldOffset(100)] public float Ceiling;               // 0.97
    // 0x68..0x80: six floats of padding. HLSL-side these are six scalars
    // (vectors cannot straddle 16-byte rows — float4 padding would land at
    // 112 and desync the whole tail).
    [FieldOffset(104)] public float Pad0;
    [FieldOffset(108)] public float Pad1;
    [FieldOffset(112)] public float Pad2;
    [FieldOffset(116)] public float Pad3;
    [FieldOffset(120)] public float Pad4;
    [FieldOffset(124)] public float Pad5;
    [FieldOffset(128)] public BackdropModel Model0;        // 0x80 identity, t=60
    [FieldOffset(208)] public BackdropModel Model1;        // translate(-0.5, 0.7), t=45
    [FieldOffset(288)] public BackdropModel Model2;        // translate(-0.95, -0.7), t=35

    public static BackdropUniforms Create(float aspect)
    {
        var uniforms = new BackdropUniforms
        {
            ViewMatrix = new Matrix4x4(
                Math.Max(1, 1 / aspect), 0, 0, 0,
                0, Math.Max(1, aspect), 0, 0,
                0, 0, 1, 0,
                0, 0, 0, 1),
            Time = 0,
            TextureTransitionMix = 0,
            MeshWarpTimeScale = 1.75f,
            Saturation = 1.3f,
            WhiteScrimAlpha = 0,
            BlackScrimAlpha = 0.25f,
            FactorForDarkMode = 0.2f,
            FactorForLightMode = 0,
            Floor = 0.07f,
            Ceiling = 0.97f,
            Model0 = new BackdropModel(Matrix4x4.Identity, 60),
            Model1 = new BackdropModel(Matrix4x4.CreateTranslation(-0.5f, 0.7f, 0), 45),
            Model2 = new BackdropModel(Matrix4x4.CreateTranslation(-0.95f, -0.7f, 0), 35),
        };
        return uniforms;
    }
}

/// <summary>
/// One warp-mesh vertex: grid position, the two Music.app control-pair
/// positions (as NDC), and the grid UV. 14 floats / 56 bytes, matching the
/// Metal <c>BackdropVertex</c> attribute layout.
/// </summary>
[StructLayout(LayoutKind.Sequential)]
public struct BackdropVertex
{
    public float PosX;
    public float PosY;
    public float PosZ;
    public float PosW;
    public float FromX;
    public float FromY;
    public float FromZ;
    public float FromW;
    public float ToX;
    public float ToY;
    public float ToZ;
    public float ToW;
    public float UvX;
    public float UvY;

    public BackdropVertex(float gridX, float gridY, float fromX, float fromY, float toX, float toY)
    {
        PosX = gridX;
        PosY = gridY;
        PosZ = 0;
        PosW = 1;
        FromX = fromX * 2 - 1;
        FromY = fromY * 2 - 1;
        FromZ = 0;
        FromW = 1;
        ToX = toX * 2 - 1;
        ToY = toY * 2 - 1;
        ToZ = 0;
        ToW = 1;
        UvX = gridX;
        UvY = gridY;
    }
}
