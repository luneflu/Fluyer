package org.alvindimas05.fluyer.state

import android.util.Log
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import org.json.JSONArray
import org.json.JSONObject
import java.io.File

/** What the animated backdrop renders: the real cover, or Rust's palette-block square. */
enum class BackdropSource(val key: String, val label: String) {
    ARTWORK("artwork", "Artwork"),
    BLOCKS("blocks", "Color blocks");

    companion object {
        fun fromKey(key: String?) = entries.firstOrNull { it.key == key } ?: ARTWORK
    }
}

/**
 * User settings persisted as JSON (`settings.json`). `file == null` keeps everything in
 * memory (tests). ponytail: no Discord setting; the core disables Discord on Android.
 */
class SettingsState(private val file: File? = null) {
    private var loading = false

    var musicFolders by mutableStateOf(listOf<String>())
        private set

    private var _animatedBackground by mutableStateOf(true)
    var animatedBackground: Boolean
        get() = _animatedBackground
        set(v) { if (v != _animatedBackground) { _animatedBackground = v; save() } }

    private var _backdropSource by mutableStateOf(BackdropSource.ARTWORK)
    var backdropSource: BackdropSource
        get() = _backdropSource
        set(v) { if (v != _backdropSource) { _backdropSource = v; save() } }

    private var _volume by mutableStateOf(1f)

    /** Last session volume; written at stop, not per slider tick. */
    var volume: Float
        get() = _volume
        set(v) { if (v != _volume) { _volume = v; save() } }

    init { load() }

    /** Adds folders not already present (trailing slash ignored). Returns the ones added. */
    fun addFolders(paths: List<String>): List<String> {
        val added = mutableListOf<String>()
        for (p in paths.map(::normalize)) {
            if (p.isNotEmpty() && p !in musicFolders && p !in added) added += p
        }
        if (added.isNotEmpty()) { musicFolders = musicFolders + added; save() }
        return added
    }

    /** Forgets a folder; returns the stored spelling (what scans used), or null if unknown. */
    fun removeFolder(path: String): String? {
        val key = normalize(path)
        if (key !in musicFolders) return null
        musicFolders = musicFolders - key
        save()
        return key
    }

    private fun load() {
        val f = file ?: return
        if (!f.exists()) return
        loading = true
        try {
            val o = JSONObject(f.readText())
            val folders = o.optJSONArray("musicFolders")
            val list = buildList { if (folders != null) for (i in 0 until folders.length()) add(normalize(folders.optString(i))) }
            musicFolders = list.filter { it.isNotEmpty() }.distinct()
            _animatedBackground = o.optBoolean("animatedBackground", true)
            _backdropSource = BackdropSource.fromKey(o.optString("backdropSource"))
            val v = o.optDouble("volume", 1.0)
            _volume = (if (v.isNaN()) 1f else v.toFloat()).clamped(0f, 1f)
        } catch (e: Exception) {
            // Unreadable file: keep a copy so the next save doesn't destroy it.
            Log.w("Fluyer", "Settings unreadable, using defaults", e)
            val bad = File(f.path + ".bad")
            runCatching { bad.delete(); f.copyTo(bad) }
        } finally {
            loading = false
        }
    }

    private fun save() {
        val f = file ?: return
        if (loading) return
        runCatching {
            val o = JSONObject()
                .put("musicFolders", JSONArray(musicFolders))
                .put("animatedBackground", _animatedBackground)
                .put("backdropSource", _backdropSource.key)
                .put("volume", _volume.toDouble())
            f.parentFile?.mkdirs()
            val tmp = File(f.path + ".tmp")
            tmp.writeText(o.toString(2))
            if (!tmp.renameTo(f)) { f.delete(); tmp.renameTo(f) }
        }.onFailure { Log.w("Fluyer", "Settings save failed", it) }
    }

    companion object {
        const val FILE_NAME = "settings.json"

        fun normalize(path: String): String {
            val t = path.trim()
            return if (t.length > 1 && t.endsWith("/")) t.trimEnd('/') else t
        }
    }
}
