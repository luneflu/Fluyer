package org.alvindimas05.fluyer.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import org.alvindimas05.fluyer.support.clamped
import uniffi.fluyer_core.FluyerAppEngineInterface
import uniffi.fluyer_core.SettingsViewModel

/** What the animated backdrop renders: the real cover, or Rust's palette-block square. */
enum class BackdropSource(val key: String, val label: String) {
    ARTWORK("artwork", "Artwork"),
    BLOCKS("blocks", "Color blocks");

    companion object {
        fun fromKey(key: String?) = entries.firstOrNull { it.key == key } ?: ARTWORK
    }
}

/**
 * User settings for the app. The core owns the file (`<filesDir>/settings.json`) and
 * every rule that used to live here: JSON shape, folder normalization, dedupe, volume
 * clamp, unknown backdrop value. This type is the observable mirror views bind to; it
 * never touches disk. A null engine keeps everything in memory (tests).
 *
 * ponytail: no Discord setting; the core disables Discord on Android.
 */
class SettingsState {
    /** Set once by [AppState] right after the core starts. */
    var engine: FluyerAppEngineInterface? = null

    private var applying = false

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

    /** No Android switch for Discord (the core disables it here); keep the stored value. */
    private var discordRpc = true

    /** Last session volume; written at stop, not per slider tick. */
    var volume: Float
        get() = _volume
        set(v) { if (v != _volume) { _volume = v; save() } }

    /** Reads the stored settings; a null engine keeps the defaults. */
    fun load() {
        val e = engine ?: return
        apply(e.settingsGet())
    }

    /** Adds folders not already present. The core normalizes and dedupes; returns what got stored. */
    fun addFolders(paths: List<String>): List<String> {
        val before = musicFolders
        musicFolders = musicFolders + paths
        save()
        return musicFolders.filter { it !in before }
    }

    /** Forgets a folder; returns the stored spelling (what scans used), or null if unknown. */
    fun removeFolder(path: String): String? {
        val key = matchingForm(path)
        val stored = musicFolders.firstOrNull { matchingForm(it) == key } ?: return null
        musicFolders = musicFolders - stored
        save()
        return stored
    }

    /** Compares a picked path with a stored one: the same trailing-separator trim the core uses. */
    private fun matchingForm(path: String): String {
        val trimmed = path.trim()
        if (trimmed.length <= 1) return trimmed
        val stripped = trimmed.trimEnd('/', '\\')
        return if (stripped.isEmpty()) "/" else stripped
    }

    private fun save() {
        val e = engine ?: return
        if (applying) return
        apply(e.settingsSet(SettingsViewModel(
            musicFolders = musicFolders,
            animatedBackground = _animatedBackground,
            backdropSource = _backdropSource.key,
            discordRpc = discordRpc,
            volume = _volume,
        )))
    }

    /** Mirrors the core's canonical values back into the observable properties. */
    private fun apply(stored: SettingsViewModel) {
        applying = true
        try {
            musicFolders = stored.musicFolders
            _animatedBackground = stored.animatedBackground
            _backdropSource = BackdropSource.fromKey(stored.backdropSource)
            discordRpc = stored.discordRpc
            _volume = stored.volume.clamped(0f, 1f)
        } finally {
            applying = false
        }
    }
}
