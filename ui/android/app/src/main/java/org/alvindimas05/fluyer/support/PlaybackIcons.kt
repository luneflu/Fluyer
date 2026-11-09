package org.alvindimas05.fluyer.support

import uniffi.fluyer_core.NativeRepeatMode

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
