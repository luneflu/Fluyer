package org.alvindimas05.fluyer.state

import uniffi.fluyer_core.ColorRgb
import uniffi.fluyer_core.NativeRepeatMode
import uniffi.fluyer_core.PlayViewModel
import uniffi.fluyer_core.PlayerBarViewModel
import uniffi.fluyer_core.ScanStatusViewModel

fun Float.clamped(lo: Float, hi: Float): Float = if (this < lo) lo else if (this > hi) hi else this

/**
 * Elapsed/duration formatting. ponytail: the core also formats `time_label`, but the
 * UI needs bare `m:ss` for the two separate labels; both live here so they cannot drift.
 */
object TimeFormat {
    /** `m:ss`, truncating sub-second precision. */
    fun elapsed(ms: ULong): String {
        val s = (ms / 1000uL).toLong()
        return "%d:%02d".format(java.util.Locale.US, s / 60, s % 60)
    }

    /** `"position / duration"`, matching the core's `time_label` layout. */
    fun pair(positionMs: ULong, durationMs: ULong) = "${elapsed(positionMs)} / ${elapsed(durationMs)}"
}

/** Icon choices driven by transport state; the UI maps them to vectors. */
object PlaybackIcons {
    enum class Repeat { REPEAT, REPEAT_ONE }
    enum class Volume { MUTE, LOW, MID, HIGH }

    fun repeat(mode: NativeRepeatMode) = if (mode == NativeRepeatMode.ONE) Repeat.REPEAT_ONE else Repeat.REPEAT

    fun volume(level: Float) = when {
        level <= 0.001f -> Volume.MUTE
        level < 0.33f -> Volume.LOW
        level < 0.66f -> Volume.MID
        else -> Volume.HIGH
    }
}

/**
 * Cache keys for [ThumbnailStore]. The requested pixel size is part of every key so a
 * 400px carousel card and an 88px row for the same album never share one bitmap.
 */
object ThumbnailKey {
    fun track(index: ULong, px: Int) = "track-$index@$px"
    fun album(index: ULong, px: Int) = "album-$index@$px"

    /** Keyed by the playing track's path so a rescan that renumbers tracks cannot serve the wrong cover. */
    fun current(side: Int, path: String) = "current-$side-$path"

    fun trackPrefix(index: ULong) = "track-$index@"
    fun albumPrefix(index: ULong) = "album-$index@"
    const val CURRENT_PREFIX = "current-"
}

/** File-manager order: case-insensitive, digit runs compared by value ("2" before "10"). */
fun naturalCompare(a: String, b: String): Int {
    var i = 0
    var j = 0
    while (i < a.length && j < b.length) {
        if (a[i].isDigit() && b[j].isDigit()) {
            var ei = i
            while (ei < a.length && a[ei].isDigit()) ei++
            var ej = j
            while (ej < b.length && b[ej].isDigit()) ej++
            val na = a.substring(i, ei).trimStart('0')
            val nb = b.substring(j, ej).trimStart('0')
            if (na.length != nb.length) return na.length.compareTo(nb.length)
            val c = na.compareTo(nb)
            if (c != 0) return c
            i = ei
            j = ej
        } else {
            val c = a[i].lowercaseChar().compareTo(b[j].lowercaseChar())
            if (c != 0) return c
            i++
            j++
        }
    }
    return (a.length - i).compareTo(b.length - j)
}

// Placeholders shown before the core reports anything; mirror the Rust constructors.
val NO_TRACK_BAR = PlayerBarViewModel(
    trackIndex = -1, title = "No Track", artist = "", album = "", positionMs = 0uL, durationMs = 0uL,
    progressPct = 0f, timeLabel = "0:00 / 0:00", isPlaying = false, repeatMode = NativeRepeatMode.NONE,
    isShuffled = false, volume = 1f,
)
val EMPTY_PLAY_VIEW = PlayViewModel(
    track = null, lyrics = emptyList(), currentLyricIndex = -1,
    palette = listOf(ColorRgb(28u.toUByte(), 28u.toUByte(), 36u.toUByte())),
)
val IDLE_SCAN = ScanStatusViewModel(
    isScanning = false, current = 0uL, total = 0uL, progressPct = 0f, statusLabel = "Ready",
)
