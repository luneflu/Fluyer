using System.Runtime.InteropServices;
using Fluyer.Core.Rendering;

namespace Fluyer.Tests;

public sealed class BackdropLayoutTests
{
    [Fact]
    public void Uniforms_Are368Bytes()
        => Assert.Equal(368, Marshal.SizeOf<BackdropUniforms>());

    [Fact]
    public void Model_Is80Bytes()
        => Assert.Equal(80, Marshal.SizeOf<BackdropModel>());

    [Fact]
    public void Vertex_Is56Bytes()
        => Assert.Equal(56, Marshal.SizeOf<BackdropVertex>());

    [Fact]
    public void FieldOffsets_MatchMetalAbi()
    {
        Assert.Equal(0, (int)Marshal.OffsetOf<BackdropUniforms>(nameof(BackdropUniforms.ViewMatrix)));
        Assert.Equal(64, (int)Marshal.OffsetOf<BackdropUniforms>(nameof(BackdropUniforms.Time)));
        Assert.Equal(68, (int)Marshal.OffsetOf<BackdropUniforms>(nameof(BackdropUniforms.TextureTransitionMix)));
        Assert.Equal(128, (int)Marshal.OffsetOf<BackdropUniforms>(nameof(BackdropUniforms.Model0)));
        Assert.Equal(208, (int)Marshal.OffsetOf<BackdropUniforms>(nameof(BackdropUniforms.Model1)));
        Assert.Equal(288, (int)Marshal.OffsetOf<BackdropUniforms>(nameof(BackdropUniforms.Model2)));
    }

    [Fact]
    public void Create_SetsDocumentedDefaults()
    {
        var u = BackdropUniforms.Create(aspect: 16.0f / 9.0f);
        Assert.Equal(1.75f, u.MeshWarpTimeScale);
        Assert.Equal(1.3f, u.Saturation);
        Assert.Equal(0.25f, u.BlackScrimAlpha);
        Assert.Equal(0.2f, u.FactorForDarkMode);
        Assert.Equal(0.07f, u.Floor);
        Assert.Equal(0.97f, u.Ceiling);
        Assert.Equal(60, u.Model0.TimeScale);
        Assert.Equal(45, u.Model1.TimeScale);
        Assert.Equal(35, u.Model2.TimeScale);
        // Landscape aspect stretches Y (portrait would stretch X).
        Assert.Equal(1, u.ViewMatrix.M11);
        Assert.Equal(16.0f / 9.0f, u.ViewMatrix.M22);
    }

    [Fact]
    public void StructureToPtr_RoundTripsLikeMapUpload()
    {
        // Mirrors ArtworkBackdropRenderer.UploadUniforms exactly: the GPU only
        // ever sees these bytes, so any drift here blanks every frame.
        var u = BackdropUniforms.Create(aspect: 1.5f);
        u.Time = 12.5f;
        var size = Marshal.SizeOf<BackdropUniforms>();
        var ptr = Marshal.AllocHGlobal(size);
        try
        {
            Marshal.StructureToPtr(u, ptr, false);
            var bytes = new byte[size];
            Marshal.Copy(ptr, bytes, 0, size);
            var floats = new float[size / 4];
            Buffer.BlockCopy(bytes, 0, floats, 0, size);
            Assert.Equal(1, floats[0]);
            Assert.Equal(1.5f, floats[5]);
            Assert.Equal(12.5f, floats[16]); // time @64
            Assert.Equal(60, floats[48]); // model0 timeScale @192
            Assert.Equal(-0.5f, floats[64]); // model1 M41 @256
        }
        finally
        {
            Marshal.FreeHGlobal(ptr);
        }
    }
    [Fact]
    public void MarshalledBytes_MatchMetalAbiValues()
    {
        // Guards the exact float stream the GPU reads: a zeroed or shifted
        // uniforms buffer collapses every vertex to w=0 and blanks the frame.
        var u = BackdropUniforms.Create(aspect: 1.5f);
        var size = Marshal.SizeOf<BackdropUniforms>();
        var bytes = new byte[size];
        var handle = GCHandle.Alloc(u, GCHandleType.Pinned);
        try
        {
            Marshal.Copy(handle.AddrOfPinnedObject(), bytes, 0, size);
        }
        finally
        {
            handle.Free();
        }
        var floats = new float[size / 4];
        Buffer.BlockCopy(bytes, 0, floats, 0, size);

        // View matrix diagonal: M11=1, M22=aspect.
        Assert.Equal(1, floats[0]);
        Assert.Equal(0, floats[1]);
        Assert.Equal(1.5f, floats[5]);
        Assert.Equal(1, floats[15]);
        // Header floats at 64..100.
        Assert.Equal(1.75f, floats[18]); // meshWarpTimeScale @72
        Assert.Equal(1.3f, floats[19]); // saturation @76
        Assert.Equal(0.07f, floats[24]); // floor @96
        Assert.Equal(0.97f, floats[25]); // ceiling @100
        // Model0 identity + timeScale 60 @128+64=192.
        Assert.Equal(1, floats[32]);
        Assert.Equal(60, floats[48]);
        // Model1 translation (-0.5, 0.7): M41 @208+48=256 -> floats[64].
        Assert.Equal(-0.5f, floats[64]);
        Assert.Equal(0.7f, floats[65]);
        Assert.Equal(45, floats[68]); // timeScale @208+64=272
    }
}

public sealed class BackdropMeshTests
{
    [Fact]
    public void Mesh_HasExpectedTopology()
    {
        var (vertices, indices) = BackdropMesh.MakeMesh();
        // 6x6 control grid, 3 subdivision rounds of 25 quads.
        Assert.True(vertices.Length > 36);
        Assert.Equal(0, indices.Length % 6);
        Assert.All(indices, i => Assert.True(i < vertices.Length));
    }

    [Fact]
    public void Mesh_GridPositionsStayInUnitSquare()
    {
        var (vertices, _) = BackdropMesh.MakeMesh();
        Assert.All(vertices, v =>
        {
            Assert.InRange(v.PosX, 0, 1);
            Assert.InRange(v.PosY, 0, 1);
            Assert.Equal(v.PosX, v.UvX);
            Assert.Equal(v.PosY, v.UvY);
        });
    }

    [Fact]
    public void Mesh_CornersArePinned()
    {
        var (vertices, _) = BackdropMesh.MakeMesh();
        var bl = vertices[0];
        Assert.Equal(0, bl.PosX);
        Assert.Equal(0, bl.PosY);
        Assert.Equal(-1, bl.FromX);
        Assert.Equal(-1, bl.FromY);
        var tr = vertices.First(v => v.PosX == 1 && v.PosY == 1);
        Assert.Equal(1, tr.FromX);
        Assert.Equal(1, tr.FromY);
    }

    [Fact]
    public void Mesh_IsDeterministic()
    {
        var (a, ai) = BackdropMesh.MakeMesh();
        var (b, bi) = BackdropMesh.MakeMesh();
        Assert.Equal(a.Length, b.Length);
        Assert.Equal(ai, bi);
        Assert.Equal(a[100].FromX, b[100].FromX);
        Assert.Equal(a[100].ToY, b[100].ToY);
    }
}
