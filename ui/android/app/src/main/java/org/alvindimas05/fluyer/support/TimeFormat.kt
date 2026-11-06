package org.alvindimas05.fluyer.support

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
