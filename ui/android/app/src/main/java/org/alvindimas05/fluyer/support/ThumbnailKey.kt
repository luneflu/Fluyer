package org.alvindimas05.fluyer.support

/**
 * Cache keys for the thumbnail store. The requested pixel size is part of every key so a
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
