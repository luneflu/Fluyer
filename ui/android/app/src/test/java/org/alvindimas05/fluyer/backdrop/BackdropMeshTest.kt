package org.alvindimas05.fluyer.backdrop

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class BackdropMeshTest {
    private val mesh = BackdropMesh.make()

    @Test fun Mesh_HasExpectedTopology() {
        // 6x6 control grid, 3 subdivision rounds of 25 quads: 25 * 4^3 = 1600 quads.
        assertTrue(mesh.vertexCount > 36)
        assertEquals(0, mesh.indices.size % 6)
        assertEquals(1600 * 6, mesh.indices.size)
        assertEquals(mesh.vertices.size, mesh.vertexCount * BackdropMesh.FLOATS_PER_VERTEX)
    }

    @Test fun Mesh_IndicesInBounds() {
        assertTrue(mesh.indices.all { it in 0 until mesh.vertexCount })
    }

    @Test fun Mesh_GridPositionsStayInUnitSquare() {
        for (i in 0 until mesh.vertexCount) {
            val x = mesh.vertices[i * 6]
            val y = mesh.vertices[i * 6 + 1]
            assertTrue(x in 0f..1f && y in 0f..1f)
        }
    }

    @Test fun Mesh_CornersArePinned() {
        // vertex 0 is control point (0,0): grid (0,0), from/to NDC (-1,-1).
        assertArrayEquals(floatArrayOf(0f, 0f, -1f, -1f, -1f, -1f), mesh.vertices.copyOfRange(0, 6), 0f)
    }

    @Test fun Mesh_IsDeterministic() {
        val again = BackdropMesh.make()
        assertArrayEquals(mesh.vertices, again.vertices, 0f)
        assertArrayEquals(mesh.indices, again.indices)
    }
}
