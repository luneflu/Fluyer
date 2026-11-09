package org.alvindimas05.fluyer.support

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.fluyer_core.NativeRepeatMode

class SupportTest {
    // TimeFormat
    @Test fun timeFormat() {
        assertEquals("0:00", TimeFormat.elapsed(0uL))
        assertEquals("1:05", TimeFormat.elapsed(65_999uL))
        assertEquals("61:01", TimeFormat.elapsed(3_661_000uL))
        assertEquals("0:01 / 3:00", TimeFormat.pair(1000uL, 180_000uL))
    }

    // PlaybackIcons
    @Test fun icons() {
        assertEquals(PlaybackIcons.Repeat.REPEAT_ONE, PlaybackIcons.repeat(NativeRepeatMode.ONE))
        assertEquals(PlaybackIcons.Repeat.REPEAT, PlaybackIcons.repeat(NativeRepeatMode.ALL))
        assertEquals(PlaybackIcons.Volume.MUTE, PlaybackIcons.volume(0f))
        assertEquals(PlaybackIcons.Volume.LOW, PlaybackIcons.volume(0.2f))
        assertEquals(PlaybackIcons.Volume.MID, PlaybackIcons.volume(0.5f))
        assertEquals(PlaybackIcons.Volume.HIGH, PlaybackIcons.volume(1f))
    }

    // ThumbnailKey
    @Test fun thumbnailKeys() {
        assertNotEquals(ThumbnailKey.album(3uL, 88), ThumbnailKey.album(3uL, 400))
        assertTrue(ThumbnailKey.album(3uL, 88).startsWith(ThumbnailKey.albumPrefix(3uL)))
        assertFalse(ThumbnailKey.album(31uL, 88).startsWith(ThumbnailKey.albumPrefix(3uL)))
        assertTrue(ThumbnailKey.track(7uL, 88).startsWith(ThumbnailKey.trackPrefix(7uL)))
        assertEquals("current-88-/a.mp3", ThumbnailKey.current(88, "/a.mp3"))
    }

    // naturalCompare
    @Test fun naturalOrder() {
        assertTrue(naturalCompare("2", "10") < 0)
        assertTrue(naturalCompare("abc", "ABD") < 0)
        assertEquals(0, naturalCompare("Same", "same"))
        assertTrue(naturalCompare("a", "ab") < 0)
    }
}
