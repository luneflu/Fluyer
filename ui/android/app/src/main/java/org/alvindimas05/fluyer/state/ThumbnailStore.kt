package org.alvindimas05.fluyer.state

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.util.LruCache
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.setValue
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/**
 * Bounded LRU of decoded thumbnails (byte-sized). Keeps only small pre-downscaled bitmaps.
 *
 * [generation] bumps on [invalidate] so composables keyed on it reload a cover that finished
 * loading after they first rendered a placeholder.
 */
class ThumbnailStore(byteLimit: Int = 24 * 1024 * 1024) {
    private val cache = object : LruCache<String, Bitmap>(byteLimit) {
        override fun sizeOf(key: String, value: Bitmap) = value.byteCount
    }

    var generation by mutableIntStateOf(0)
        private set

    fun peek(key: String): Bitmap? = synchronized(cache) { cache.get(key) }

    /** Drop every cached size of one cover. */
    fun invalidate(prefix: String) {
        synchronized(cache) {
            cache.snapshot().keys.filter { it.startsWith(prefix) }.forEach { cache.remove(it) }
        }
        generation++
    }

    /** Decode JPEG bytes off the main thread (uncached; for one-off backdrop images). */
    suspend fun decode(data: ByteArray): Bitmap? =
        withContext(Dispatchers.Default) { BitmapFactory.decodeByteArray(data, 0, data.size) }

    /** RGBA (noneSkipLast) pixels to an ARGB_8888 bitmap, off the main thread. */
    suspend fun rgbaToBitmap(rgba: ByteArray, w: Int, h: Int): Bitmap = withContext(Dispatchers.Default) {
        val px = IntArray(w * h) { i ->
            val o = i * 4
            (0xFF shl 24) or ((rgba[o].toInt() and 0xFF) shl 16) or ((rgba[o + 1].toInt() and 0xFF) shl 8) or (rgba[o + 2].toInt() and 0xFF)
        }
        Bitmap.createBitmap(px, w, h, Bitmap.Config.ARGB_8888)
    }

    /** Cached bitmap, else fetch JPEG bytes via [load] and decode off the main thread. */
    suspend fun image(key: String, load: suspend () -> ByteArray?): Bitmap? {
        peek(key)?.let { return it }
        // ponytail: core futures run synchronously on the polling thread, so a `load*` call from a
        // main-dispatcher coroutine would block the UI; always poll them on Default.
        val data = withContext(Dispatchers.Default) { load() }?.takeIf { it.isNotEmpty() } ?: return null
        val bmp = withContext(Dispatchers.Default) { BitmapFactory.decodeByteArray(data, 0, data.size) } ?: return null
        synchronized(cache) { cache.put(key, bmp) }
        return bmp
    }
}
