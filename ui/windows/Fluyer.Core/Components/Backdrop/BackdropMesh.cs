namespace Fluyer.Core.Rendering;

/// <summary>
/// Line-by-line port of <c>ArtworkBackdropRenderer.makeMesh()</c> from
/// <c>ui/macos/Sources/Components/Backdrop/ArtworkBackdropRenderer.swift</c>:
/// the 6x6 Music.app warp control grid, subdivided 3x with a public
/// Catmull-Clark approximation of the private CAMeshTransform subdividedMesh:3.
/// One recovered control pair (index 10 and 13 differ between from/to); the
/// other four are added only after visual comparison — same as macOS.
/// </summary>
public static class BackdropMesh
{
    // Control-pair 0 at Music arm64e 0x101885858. Five divisions, six points per axis.
    private static readonly float[][] From =
    [
        [0, 0], [0.2f, 0], [0.4f, 0], [0.6f, 0], [0.8f, 0], [1, 0],
        [0, 0.2f], [-0.0933f, 0.4f], [0.4f, 0.2f], [0.6f, 0.2f], [0.3653f, 0.1335f], [1, 0.2f],
        [0, 0.4f], [0.4232f, 0.359f], [0.3429f, 0.5349f], [0.6f, 0.4f], [0.832f, 0.4148f], [1, 0.4f],
        [0, 0.6f], [0.2f, 0.6f], [0.2293f, 0.7775f], [0.7829f, 0.5595f], [0.6514f, 0.7302f], [1, 0.6f],
        [0, 0.8f], [0.2f, 0.8f], [0.28f, 0.9195f], [0.4773f, 0.8f], [0.8f, 0.8f], [1, 0.8f],
        [0, 1], [0.6514f, 1.1073f], [0.4f, 1], [1, 1.0317f], [1, 1.1302f], [1, 1],
    ];

    public static (BackdropVertex[] Vertices, uint[] Indices) MakeMesh()
    {
        var to = From.Select(p => new[] { p[0], p[1] }).ToArray();
        to[10] = [0.8587f, 0.2234f];
        to[13] = [0.4526f, 0.6053f];

        // points[i] = [gridX, gridY, fromX, fromY, toX, toY, 0, 0]
        var points = new List<float[]>();
        for (var i = 0; i < 36; i++)
        {
            points.Add([
                (i % 6) / 5.0f, (i / 6) / 5.0f,
                From[i][0], From[i][1], to[i][0], to[i][1], 0, 0,
            ]);
        }

        var faces = new List<int[]>();
        for (var y = 0; y < 5; y++)
        {
            for (var x = 0; x < 5; x++)
            {
                var a = y * 6 + x;
                faces.Add([a, a + 1, a + 7, a + 6]);
            }
        }

        for (var iter = 0; iter < 3; iter++)
        {
            var edges = new List<Edge>();
            var edgeIds = new Dictionary<ulong, int>();
            var faceEdges = Enumerable.Range(0, faces.Count).Select(_ => new List<int>()).ToList();
            var vertexFaces = Enumerable.Range(0, points.Count).Select(_ => new List<int>()).ToList();
            var vertexEdges = Enumerable.Range(0, points.Count).Select(_ => new List<int>()).ToList();
            var centers = faces
                .Select(face =>
                {
                    var sum = new float[8];
                    foreach (var vi in face)
                    {
                        AddInto(sum, points[vi]);
                    }
                    return Div(sum, 4);
                })
                .ToList();

            for (var fi = 0; fi < faces.Count; fi++)
            {
                var face = faces[fi];
                for (var j = 0; j < 4; j++)
                {
                    var a = face[j];
                    var b = face[(j + 1) % 4];
                    vertexFaces[a].Add(fi);
                    var key = ((ulong)(uint)Math.Min(a, b) << 32) | (uint)Math.Max(a, b);
                    int ei;
                    if (edgeIds.TryGetValue(key, out var existing))
                    {
                        ei = existing;
                        edges[ei].Faces.Add(fi);
                    }
                    else
                    {
                        ei = edges.Count;
                        edgeIds[key] = ei;
                        edges.Add(new Edge(a, b, [fi]));
                        vertexEdges[a].Add(ei);
                        vertexEdges[b].Add(ei);
                    }
                    faceEdges[fi].Add(ei);
                }
            }

            var refined = points.Select(p => (float[])p.Clone()).ToList();
            for (var i = 0; i < points.Count; i++)
            {
                var boundary = vertexEdges[i].Where(ei => edges[ei].Faces.Count == 1).ToList();
                if (boundary.Count == 2 && vertexFaces[i].Count > 1)
                {
                    var n0 = edges[boundary[0]].A == i ? edges[boundary[0]].B : edges[boundary[0]].A;
                    var n1 = edges[boundary[1]].A == i ? edges[boundary[1]].B : edges[boundary[1]].A;
                    refined[i] = Add(Mul(points[i], 0.75f), Mul(Add(points[n0], points[n1]), 0.125f));
                }
                else if (boundary.Count == 0)
                {
                    var n = (float)vertexEdges[i].Count;
                    var f = Div(vertexFaces[i].Aggregate(new float[8], (acc, fi2) => AddInto(acc, centers[fi2])), n);
                    var r = Div(vertexEdges[i].Aggregate(new float[8], (acc, ei2) =>
                        AddInto(acc, Mul(Add(points[edges[ei2].A], points[edges[ei2].B]), 0.5f))), n);
                    refined[i] = Div(Add(Add(f, Mul(r, 2)), Mul(points[i], n - 3)), n);
                }
                refined[i][0] = points[i][0];
                refined[i][1] = points[i][1];
            }

            var edgeBase = refined.Count;
            foreach (var edge in edges)
            {
                var midpoint = Mul(Add(points[edge.A], points[edge.B]), 0.5f);
                var p = (float[])midpoint.Clone();
                if (edge.Faces.Count == 2)
                {
                    p = Mul(Add(Add(points[edge.A], points[edge.B]),
                        Add(centers[edge.Faces[0]], centers[edge.Faces[1]])), 0.25f);
                }
                p[0] = midpoint[0];
                p[1] = midpoint[1];
                refined.Add(p);
            }

            var faceBase = refined.Count;
            refined.AddRange(centers);

            var subdivided = new List<int[]>();
            for (var fi = 0; fi < faces.Count; fi++)
            {
                var face = faces[fi];
                for (var j = 0; j < 4; j++)
                {
                    subdivided.Add([
                        face[j],
                        edgeBase + faceEdges[fi][j],
                        faceBase + fi,
                        edgeBase + faceEdges[fi][(j + 3) % 4],
                    ]);
                }
            }

            points = refined;
            faces = subdivided;
        }

        var vertices = points
            .Select(p => new BackdropVertex(p[0], p[1], p[2], p[3], p[4], p[5]))
            .ToArray();
        var indices = faces
            .SelectMany(f => new uint[] { (uint)f[0], (uint)f[2], (uint)f[3], (uint)f[0], (uint)f[1], (uint)f[2] })
            .ToArray();
        return (vertices, indices);
    }

    private sealed class Edge(int a, int b, List<int> faces)
    {
        public int A = a;
        public int B = b;
        public List<int> Faces = faces;
    }

    private static float[] Add(float[] a, float[] b)
    {
        var r = new float[8];
        for (var i = 0; i < 8; i++)
        {
            r[i] = a[i] + b[i];
        }
        return r;
    }

    private static float[] AddInto(float[] acc, float[] v)
    {
        for (var i = 0; i < 8; i++)
        {
            acc[i] += v[i];
        }
        return acc;
    }

    private static float[] Mul(float[] v, float s)
    {
        var r = new float[8];
        for (var i = 0; i < 8; i++)
        {
            r[i] = v[i] * s;
        }
        return r;
    }

    private static float[] Div(float[] v, float s)
    {
        var r = new float[8];
        for (var i = 0; i < 8; i++)
        {
            r[i] = v[i] / s;
        }
        return r;
    }
}
