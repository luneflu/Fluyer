package org.alvindimas05.fluyer.backdrop

/**
 * Port of ArtworkBackdropRenderer.makeMesh() (macOS) / BackdropMesh.cs (Windows): 6x6 Music.app warp
 * control grid, subdivided 3x with a public Catmull-Clark approximation of CAMeshTransform.
 * [vertices] is interleaved (posX, posY, fromX, fromY, toX, toY), [FLOATS_PER_VERTEX] floats each;
 * pos is the 0..1 grid (also the uv), from/to are NDC (x*2-1).
 */
class BackdropMesh(val vertices: FloatArray, val indices: IntArray) {
    val vertexCount get() = vertices.size / FLOATS_PER_VERTEX

    companion object {
        const val FLOATS_PER_VERTEX = 6

        // Control-pair 0 at Music arm64e 0x101885858. Five divisions, six points per axis.
        private val FROM = floatArrayOf(
            0f, 0f, 0.2f, 0f, 0.4f, 0f, 0.6f, 0f, 0.8f, 0f, 1f, 0f,
            0f, 0.2f, -0.0933f, 0.4f, 0.4f, 0.2f, 0.6f, 0.2f, 0.3653f, 0.1335f, 1f, 0.2f,
            0f, 0.4f, 0.4232f, 0.359f, 0.3429f, 0.5349f, 0.6f, 0.4f, 0.832f, 0.4148f, 1f, 0.4f,
            0f, 0.6f, 0.2f, 0.6f, 0.2293f, 0.7775f, 0.7829f, 0.5595f, 0.6514f, 0.7302f, 1f, 0.6f,
            0f, 0.8f, 0.2f, 0.8f, 0.28f, 0.9195f, 0.4773f, 0.8f, 0.8f, 0.8f, 1f, 0.8f,
            0f, 1f, 0.6514f, 1.1073f, 0.4f, 1f, 1f, 1.0317f, 1f, 1.1302f, 1f, 1f,
        )

        private class Edge(val a: Int, val b: Int, val faces: MutableList<Int>)

        private fun add(a: FloatArray, b: FloatArray) = FloatArray(8) { a[it] + b[it] }
        private fun mul(a: FloatArray, s: Float) = FloatArray(8) { a[it] * s }

        fun make(): BackdropMesh {
            val to = FROM.copyOf()
            to[10 * 2] = 0.8587f; to[10 * 2 + 1] = 0.2234f
            to[13 * 2] = 0.4526f; to[13 * 2 + 1] = 0.6053f
            // point = [gridX, gridY, fromX, fromY, toX, toY, 0, 0]
            var points = List(36) { i ->
                floatArrayOf((i % 6) / 5f, (i / 6) / 5f, FROM[i * 2], FROM[i * 2 + 1], to[i * 2], to[i * 2 + 1], 0f, 0f)
            }
            var faces = List(25) { k -> val a = (k / 5) * 6 + k % 5; intArrayOf(a, a + 1, a + 7, a + 6) }

            repeat(3) {
                val edges = ArrayList<Edge>()
                val edgeIds = HashMap<Long, Int>()
                val faceEdges = List(faces.size) { ArrayList<Int>() }
                val vertexFaces = List(points.size) { ArrayList<Int>() }
                val vertexEdges = List(points.size) { ArrayList<Int>() }
                val centers = faces.map { f -> mul(f.fold(FloatArray(8)) { acc, v -> add(acc, points[v]) }, 0.25f) }
                for ((fi, face) in faces.withIndex()) {
                    for (j in 0 until 4) {
                        val a = face[j]
                        val b = face[(j + 1) % 4]
                        vertexFaces[a].add(fi)
                        val key = (minOf(a, b).toLong() shl 32) or maxOf(a, b).toLong()
                        val ei = edgeIds[key]?.also { edges[it].faces.add(fi) } ?: run {
                            val n = edges.size
                            edgeIds[key] = n
                            edges.add(Edge(a, b, mutableListOf(fi)))
                            vertexEdges[a].add(n)
                            vertexEdges[b].add(n)
                            n
                        }
                        faceEdges[fi].add(ei)
                    }
                }
                val refined = points.mapTo(ArrayList()) { it.copyOf() }
                for (i in points.indices) {
                    val boundary = vertexEdges[i].filter { edges[it].faces.size == 1 }
                    if (boundary.size == 2 && vertexFaces[i].size > 1) {
                        val nb = boundary.map { if (edges[it].a == i) edges[it].b else edges[it].a }
                        refined[i] = add(mul(points[i], 0.75f), mul(add(points[nb[0]], points[nb[1]]), 0.125f))
                    } else if (boundary.isEmpty()) {
                        val n = vertexEdges[i].size.toFloat()
                        val f = mul(vertexFaces[i].fold(FloatArray(8)) { acc, fi -> add(acc, centers[fi]) }, 1 / n)
                        val r = mul(vertexEdges[i].fold(FloatArray(8)) { acc, ei ->
                            add(acc, mul(add(points[edges[ei].a], points[edges[ei].b]), 0.5f))
                        }, 1 / n)
                        refined[i] = mul(add(add(f, mul(r, 2f)), mul(points[i], n - 3)), 1 / n)
                    }
                    refined[i][0] = points[i][0]
                    refined[i][1] = points[i][1]
                }
                val edgeBase = refined.size
                for (e in edges) {
                    val mid = mul(add(points[e.a], points[e.b]), 0.5f)
                    val p = if (e.faces.size == 2) {
                        mul(add(add(points[e.a], points[e.b]), add(centers[e.faces[0]], centers[e.faces[1]])), 0.25f)
                    } else mid.copyOf()
                    p[0] = mid[0]; p[1] = mid[1]
                    refined.add(p)
                }
                val faceBase = refined.size
                refined.addAll(centers)
                faces = faces.flatMapIndexed { fi, f ->
                    List(4) { j -> intArrayOf(f[j], edgeBase + faceEdges[fi][j], faceBase + fi, edgeBase + faceEdges[fi][(j + 3) % 4]) }
                }
                points = refined
            }

            val vertices = FloatArray(points.size * FLOATS_PER_VERTEX)
            points.forEachIndexed { i, p ->
                val o = i * FLOATS_PER_VERTEX
                vertices[o] = p[0]; vertices[o + 1] = p[1]
                vertices[o + 2] = p[2] * 2 - 1; vertices[o + 3] = p[3] * 2 - 1
                vertices[o + 4] = p[4] * 2 - 1; vertices[o + 5] = p[5] * 2 - 1
            }
            val indices = IntArray(faces.size * 6)
            faces.forEachIndexed { i, f ->
                intArrayOf(f[0], f[2], f[3], f[0], f[1], f[2]).copyInto(indices, i * 6)
            }
            return BackdropMesh(vertices, indices)
        }
    }
}
