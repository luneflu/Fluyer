package org.alvindimas05.fluyer.backdrop

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Path
import android.graphics.RectF
import android.opengl.GLES30.*
import android.opengl.GLSurfaceView
import android.opengl.GLUtils
import android.util.Log
import java.nio.ByteBuffer
import java.nio.ByteOrder
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10
import kotlin.math.cos
import kotlin.math.hypot
import kotlin.math.ceil
import kotlin.math.floor
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sin

private const val TAG = "Backdrop"
private const val MAX_EDGE = 960      // ponytail: intermediate long-edge cap, same as macOS/Windows
private const val FRAME_NANOS = 33_000_000L // ~30 fps

private const val FULLSCREEN_VS = """#version 300 es
out vec2 uv;
void main() {
    vec2 p = vec2(gl_VertexID == 1 ? 3.0 : -1.0, gl_VertexID == 2 ? 3.0 : -1.0);
    uv = p * 0.5 + 0.5;
    gl_Position = vec4(p, 0.0, 1.0);
}"""

private const val ROTATION_VS = """#version 300 es
uniform mat4 viewMatrix;
uniform mat4 modelMtx[3];
uniform float modelTimeScale[3];
uniform float time;
out vec2 uv;
mat4 rotation(float a) {
    float c = cos(a), s = sin(a);
    return mat4(vec4(c, -s, 0, 0), vec4(s, c, 0, 0), vec4(0, 0, 1, 0), vec4(0, 0, 0, 1));
}
void main() {
    vec2 pos[6] = vec2[](vec2(-1,-1), vec2(-1,1), vec2(1,1), vec2(-1,-1), vec2(1,1), vec2(1,-1));
    vec2 tex[6] = vec2[](vec2(0,1), vec2(0,0), vec2(1,0), vec2(0,1), vec2(1,0), vec2(1,1));
    mat4 r = rotation(time * 6.28318530718 / modelTimeScale[gl_InstanceID]);
    gl_Position = viewMatrix * r * modelMtx[gl_InstanceID] * r * vec4(pos[gl_VertexID], 0.0, 1.0);
    uv = tex[gl_VertexID];
}"""

private const val COMMON_FS = """#version 300 es
precision mediump float;
in vec2 uv;
out vec4 color;
vec3 saturateColor(vec3 rgb, float amount) {
    float luma = dot(rgb, vec3(0.3, 0.59, 0.11));
    return luma + (rgb - luma) * amount;
}
"""

private const val ROTATION_FS = COMMON_FS + """
uniform sampler2D current;
uniform sampler2D previous;
uniform float transitionMix;
uniform float saturation;
void main() {
    vec3 rgb = mix(texture(current, uv).rgb, texture(previous, uv).rgb, transitionMix);
    color = vec4(saturateColor(rgb, saturation), 1.0);
}"""

// 4x4 box downsample: four bilinear taps on texel corners.
private const val DOWN4_FS = COMMON_FS + """
uniform sampler2D src;
uniform vec2 step;
void main() {
    vec3 c = texture(src, uv + vec2(-step.x, -step.y)).rgb + texture(src, uv + vec2(step.x, -step.y)).rgb
           + texture(src, uv + vec2(-step.x, step.y)).rgb + texture(src, uv + vec2(step.x, step.y)).rgb;
    color = vec4(c * 0.25, 1.0);
}"""

private const val GAUSS_FS = COMMON_FS + """
uniform sampler2D src;
uniform vec2 step;   // one blur texel along the axis, in uv
uniform float sigma; // blur texels
void main() {
    int r = min(int(ceil(sigma * 3.0)), 64);
    float inv = -0.5 / (sigma * sigma);
    vec3 sum = vec3(0.0);
    float wsum = 0.0;
    for (int k = -r; k <= r; k++) {
        float w = exp(float(k * k) * inv);
        sum += texture(src, uv + step * float(k)).rgb * w;
        wsum += w;
    }
    color = vec4(sum / wsum, 1.0);
}"""

private const val PINCH_VS = """#version 300 es
layout(location = 0) in vec2 meshUv;
layout(location = 1) in vec2 fromPos;
layout(location = 2) in vec2 toPos;
uniform float time;
uniform float warpScale;
out vec2 uv;
void main() {
    float phase = acos(sin(time * 3.14159265359 / warpScale)) / 3.14159265359;
    float blend = phase * phase * (3.0 - 2.0 * phase);
    gl_Position = vec4(mix(fromPos, toPos, blend), 0.0, 1.0);
    // The blurred texture is a GL framebuffer (row 0 at bottom); Metal's had row 0 at top.
    vec2 t = (meshUv - 0.5) * 0.8 + 0.5;
    uv = vec2(t.x, 1.0 - t.y);
}"""

private const val PINCH_FS = COMMON_FS + """
uniform sampler2D blurred;
uniform float saturation;
void main() {
    vec3 rgb = min(saturateColor(texture(blurred, uv).rgb, saturation), vec3(0.995));
    // Dark mode only: blackScrimAlpha 0.25, factorForDarkMode 0.2.
    rgb = mix(rgb, vec3(0.0), 0.25) - 0.2 * 0.1;
    color = vec4(clamp(rgb, vec3(0.07), vec3(0.97)), 1.0);
}"""

private class Target(val tex: Int, val fbo: Int, val w: Int, val h: Int)

/** Port of ArtworkBackdropRenderer (Metal): rotation -> blur -> pinch mesh. All GL calls on the GL thread. */
internal class BackdropRenderer(private val onError: () -> Unit) : GLSurfaceView.Renderer {
    @Volatile var reduceMotion = false
    private class Request(val bitmap: Bitmap?)
    @Volatile private var requested: Request? = null
    private var applied: Request? = null
    // Last applied bitmap, re-uploaded when the EGL context is lost on pause.
    private var artwork: Bitmap? = null

    private var broken = false
    private var rotationProg = 0; private var down4Prog = 0; private var gaussProg = 0; private var pinchProg = 0
    private var vao = 0; private var emptyVao = 0; private var meshVbo = 0; private var meshIbo = 0
    private var indexCount = 0
    private var current = 0; private var previous = 0; private var pending = 0
    private var transitionStart = -1L
    private var placeholder = 0
    private var rotationTarget: Target? = null
    private var blurA: Target? = null
    private var blurB: Target? = null
    private var sigma = 1f
    private var width = 0; private var height = 0

    private var animationTime = 0f
    private var lastFrame = 0L

    fun setArtwork(bitmap: Bitmap?) { requested = Request(bitmap) }

    override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
        try {
            rotationProg = program(ROTATION_VS, ROTATION_FS)
            down4Prog = program(FULLSCREEN_VS, DOWN4_FS)
            gaussProg = program(FULLSCREEN_VS, GAUSS_FS)
            pinchProg = program(PINCH_VS, PINCH_FS)
            val mesh = BackdropMesh.make()
            val ids = IntArray(4)
            glGenBuffers(2, ids, 0); meshVbo = ids[0]; meshIbo = ids[1]
            glGenVertexArrays(2, ids, 2); vao = ids[2]; emptyVao = ids[3]
            glBindVertexArray(vao)
            glBindBuffer(GL_ARRAY_BUFFER, meshVbo)
            glBufferData(GL_ARRAY_BUFFER, mesh.vertices.size * 4, floatBuffer(mesh.vertices), GL_STATIC_DRAW)
            val stride = BackdropMesh.FLOATS_PER_VERTEX * 4
            for (i in 0..2) { // grid/uv, from, to
                glEnableVertexAttribArray(i)
                glVertexAttribPointer(i, 2, GL_FLOAT, false, stride, i * 8)
            }
            glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, meshIbo)
            val ib = ByteBuffer.allocateDirect(mesh.indices.size * 4).order(ByteOrder.nativeOrder())
            ib.asIntBuffer().put(mesh.indices)
            glBufferData(GL_ELEMENT_ARRAY_BUFFER, mesh.indices.size * 4, ib, GL_STATIC_DRAW)
            glBindVertexArray(0)
            indexCount = mesh.indices.size

            // GL objects from the lost context are gone; rebuild textures and render targets.
            placeholder = uploadBitmap(makePlaceholder())
            current = if (artwork != null) uploadBitmap(artwork!!) else placeholder
            previous = 0; pending = 0; transitionStart = -1L
            rotationTarget = null; blurA = null; blurB = null
            lastFrame = 0L
            broken = false
        } catch (e: RuntimeException) {
            Log.e(TAG, "GL init failed, falling back to static image", e)
            broken = true
            onError()
        }
    }

    override fun onSurfaceChanged(gl: GL10?, w: Int, h: Int) {
        width = w; height = h
        if (broken || w <= 0 || h <= 0) return
        rotationTarget?.let { release(it) }; blurA?.let { release(it) }; blurB?.let { release(it) }
        val scale = min(1.0, MAX_EDGE.toDouble() / max(w, h))
        val iw = max(1, (w * scale).toInt()); val ih = max(1, (h * scale).toInt())
        val bw = max(1, iw / 4); val bh = max(1, ih / 4)
        rotationTarget = target(iw, ih); blurA = target(bw, bh); blurB = target(bw, bh)
        // MPS sigma ported verbatim, then scaled into quarter-res blur texels.
        val sigmaIntermediate = floor(hypot(w.toDouble(), h.toDouble()) * 0.045394707) * scale
        sigma = max(sigmaIntermediate * bw / iw, 0.5).toFloat()
    }

    override fun onDrawFrame(gl: GL10?) {
        if (broken || width <= 0) return
        val now = System.nanoTime()
        if (lastFrame != 0L) {
            val wait = FRAME_NANOS - (now - lastFrame)
            if (wait > 1_000_000) try { Thread.sleep(wait / 1_000_000) } catch (_: InterruptedException) {}
        }
        val t = System.nanoTime()
        // Delta capped at 1/15 s: a pause (lastFrame stale) never jumps the animation phase.
        val delta = if (lastFrame == 0L) 0.0 else min((t - lastFrame) / 1e9, 1.0 / 15)
        animationTime += (delta / (if (reduceMotion) 10 else 1)).toFloat()
        lastFrame = t
        try { draw(t) } catch (e: RuntimeException) {
            Log.e(TAG, "frame failed", e); broken = true; onError()
        }
    }

    private fun draw(clock: Long) {
        applyRequest()
        val rot = rotationTarget!!; val a = blurA!!; val b = blurB!!
        if (transitionStart >= 0 && clock - transitionStart >= 500_000_000L) {
            if (previous != placeholder) glDeleteTextures(1, intArrayOf(previous), 0)
            previous = 0; transitionStart = -1L
        }
        if (transitionStart < 0 && pending != 0) {
            previous = current; current = pending; pending = 0; transitionStart = clock
        }
        val mix = if (transitionStart >= 0) max(0.0, 1 - (clock - transitionStart) / 5e8).toFloat() else 0f

        // Pass 1: three rotating quads into the intermediate.
        glBindFramebuffer(GL_FRAMEBUFFER, rot.fbo)
        glViewport(0, 0, rot.w, rot.h)
        glClearColor(0.07f, 0.07f, 0.09f, 1f); glClear(GL_COLOR_BUFFER_BIT)
        glUseProgram(rotationProg)
        val aspect = width.toFloat() / height
        val view = identity().also { it[0] = max(1f, 1 / aspect); it[5] = max(1f, aspect) }
        glUniformMatrix4fv(loc(rotationProg, "viewMatrix"), 1, false, view, 0)
        val models = FloatArray(48)
        for (i in 0..2) identity().copyInto(models, i * 16)
        models[16 + 12] = -0.5f; models[16 + 13] = 0.7f    // model1
        models[32 + 12] = -0.95f; models[32 + 13] = -0.7f  // model2
        glUniformMatrix4fv(loc(rotationProg, "modelMtx[0]"), 3, false, models, 0)
        glUniform1fv(loc(rotationProg, "modelTimeScale[0]"), 3, floatArrayOf(60f, 45f, 35f), 0)
        glUniform1f(loc(rotationProg, "time"), animationTime)
        glUniform1f(loc(rotationProg, "transitionMix"), mix)
        glUniform1f(loc(rotationProg, "saturation"), 1.3f)
        bind(0, current, rotationProg, "current")
        bind(1, if (previous != 0) previous else current, rotationProg, "previous")
        glBindVertexArray(emptyVao)
        glDrawArraysInstanced(GL_TRIANGLES, 0, 6, 3)

        // Pass 2: down4 + separable gaussian.
        blit(down4Prog, a, rot.tex, 1f / rot.w, 1f / rot.h)
        blit(gaussProg, b, a.tex, 1f / a.w, 0f)
        blit(gaussProg, a, b.tex, 0f, 1f / a.h)

        // Pass 3: pinch mesh to screen.
        glBindFramebuffer(GL_FRAMEBUFFER, 0)
        glViewport(0, 0, width, height)
        glClearColor(0.07f, 0.07f, 0.09f, 1f); glClear(GL_COLOR_BUFFER_BIT)
        glUseProgram(pinchProg)
        glUniform1f(loc(pinchProg, "time"), animationTime)
        glUniform1f(loc(pinchProg, "warpScale"), 1.75f)
        glUniform1f(loc(pinchProg, "saturation"), 2f)
        bind(0, a.tex, pinchProg, "blurred")
        glBindVertexArray(vao)
        glDrawElements(GL_TRIANGLES, indexCount, GL_UNSIGNED_INT, 0)
        glBindVertexArray(0)
    }

    private fun applyRequest() {
        val req = requested ?: return
        if (req === applied) return
        applied = req
        val bmp = req.bitmap
        if (bmp == artwork && artwork != null) return
        artwork = bmp
        val tex = if (bmp != null) {
            try { uploadBitmap(bmp) } catch (e: RuntimeException) { Log.w(TAG, "artwork upload failed", e); return }
        } else placeholder
        // A newer image replaces one still waiting for the crossfade slot.
        if (pending != 0 && pending != placeholder) glDeleteTextures(1, intArrayOf(pending), 0)
        pending = tex
    }

    private fun blit(prog: Int, to: Target, src: Int, sx: Float, sy: Float) {
        glBindFramebuffer(GL_FRAMEBUFFER, to.fbo)
        glViewport(0, 0, to.w, to.h)
        glUseProgram(prog)
        glUniform2f(loc(prog, "step"), sx, sy)
        glUniform1f(loc(prog, "sigma"), sigma)
        bind(0, src, prog, "src")
        glBindVertexArray(emptyVao)
        glDrawArrays(GL_TRIANGLES, 0, 3)
    }

    private fun bind(unit: Int, tex: Int, prog: Int, name: String) {
        glActiveTexture(GL_TEXTURE0 + unit)
        glBindTexture(GL_TEXTURE_2D, tex)
        glUniform1i(loc(prog, name), unit)
    }

    // ponytail: uniform lookups per frame; ~20 calls, cache in a map if profiling shows driver cost.
    private fun loc(prog: Int, name: String) = glGetUniformLocation(prog, name)

    private fun identity() = FloatArray(16).also { it[0] = 1f; it[5] = 1f; it[10] = 1f; it[15] = 1f }

    private fun floatBuffer(a: FloatArray) =
        ByteBuffer.allocateDirect(a.size * 4).order(ByteOrder.nativeOrder()).also { it.asFloatBuffer().put(a) }

    private fun target(w: Int, h: Int): Target {
        val ids = IntArray(1)
        glGenTextures(1, ids, 0)
        val tex = ids[0]
        glBindTexture(GL_TEXTURE_2D, tex)
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, w, h, 0, GL_RGBA, GL_UNSIGNED_BYTE, null)
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR)
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR)
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE)
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE)
        glGenFramebuffers(1, ids, 0)
        glBindFramebuffer(GL_FRAMEBUFFER, ids[0])
        glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, tex, 0)
        check(glCheckFramebufferStatus(GL_FRAMEBUFFER) == GL_FRAMEBUFFER_COMPLETE) { "framebuffer incomplete" }
        return Target(tex, ids[0], w, h)
    }

    private fun release(t: Target) {
        glDeleteTextures(1, intArrayOf(t.tex), 0)
        glDeleteFramebuffers(1, intArrayOf(t.fbo), 0)
    }

    private fun uploadBitmap(src: Bitmap): Int {
        // GLUtils cannot read hardware bitmaps; copy (the caller's bitmap is never touched or recycled).
        val bmp = if (src.config == Bitmap.Config.HARDWARE) src.copy(Bitmap.Config.ARGB_8888, false) else src
        val ids = IntArray(1)
        glGenTextures(1, ids, 0)
        glBindTexture(GL_TEXTURE_2D, ids[0])
        GLUtils.texImage2D(GL_TEXTURE_2D, 0, bmp, 0)
        glGenerateMipmap(GL_TEXTURE_2D) // artwork is downscaled onto <=960px; mips avoid aliasing
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR)
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR)
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE)
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE)
        if (bmp !== src) bmp.recycle()
        return ids[0]
    }

    /** 300x300 rgb(51,51,54) card with a lighter note, like Music's DefaultArtwork. */
    private fun makePlaceholder(): Bitmap {
        val side = 300
        val bmp = Bitmap.createBitmap(side, side, Bitmap.Config.ARGB_8888)
        val c = Canvas(bmp)
        c.drawColor(android.graphics.Color.rgb(51, 51, 54))
        val p = Paint(Paint.ANTI_ALIAS_FLAG).apply { color = android.graphics.Color.rgb(100, 100, 104) }
        c.drawOval(RectF(105f, 170f, 160f, 210f), p)  // head
        c.drawRect(150f, 85f, 160f, 190f, p)           // stem
        c.drawPath(Path().apply {                      // flag
            moveTo(160f, 85f); cubicTo(160f, 110f, 195f, 115f, 195f, 150f)
            cubicTo(185f, 130f, 170f, 128f, 160f, 128f); close()
        }, p)
        return bmp
    }

    private fun program(vs: String, fs: String): Int {
        fun shader(type: Int, src: String): Int {
            val s = glCreateShader(type)
            glShaderSource(s, src); glCompileShader(s)
            val ok = IntArray(1)
            glGetShaderiv(s, GL_COMPILE_STATUS, ok, 0)
            if (ok[0] == 0) throw RuntimeException("shader: " + glGetShaderInfoLog(s))
            return s
        }
        val p = glCreateProgram()
        glAttachShader(p, shader(GL_VERTEX_SHADER, vs)); glAttachShader(p, shader(GL_FRAGMENT_SHADER, fs))
        glLinkProgram(p)
        val ok = IntArray(1)
        glGetProgramiv(p, GL_LINK_STATUS, ok, 0)
        if (ok[0] == 0) throw RuntimeException("link: " + glGetProgramInfoLog(p))
        return p
    }
}
