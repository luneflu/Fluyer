package org.alvindimas05.fluyer.state

import org.junit.Assert.*
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import uniffi.fluyer_core.AlbumCardViewModel
import uniffi.fluyer_core.NativeRepeatMode
import uniffi.fluyer_core.TrackItemViewModel
import java.io.File

class StateTest {
    @get:Rule val tmp = TemporaryFolder()

    private fun track(i: Int, title: String, artist: String = "A", album: String = "Al", ms: ULong = 1000uL) =
        TrackItemViewModel(i.toULong(), "/p/$i", title, artist, album, ms, "", false)

    private fun album(i: Int, name: String, artist: String = "A", year: String = "2000", n: ULong = 1uL) =
        AlbumCardViewModel(i.toULong(), name, artist, year, n, "")

    // QueueState
    @Test fun queueTarget() {
        assertEquals(2, QueueState.target(from = 0, dropOffset = 3))
        assertEquals(0, QueueState.target(from = 2, dropOffset = 0))
        assertEquals(1, QueueState.target(from = 1, dropOffset = 1))
        assertEquals(1, QueueState.target(from = 1, dropOffset = 2))
    }

    @Test fun queueOutOfRangeNoOps() {
        val q = QueueState()
        q.remove(0); q.goto(0); q.move(0, 1); q.clear() // no engine, empty: must not throw
        assertTrue(q.tracks.isEmpty())
    }

    // AlbumSelection sort / filter
    @Test fun trackSortTiesKeepLibraryOrder() {
        val a = track(1, "x", ms = 5uL)
        val b = track(0, "x", ms = 5uL)
        assertTrue(AlbumSelection.compareTracks(b, a, TrackSort.TITLE) < 0)
        assertTrue(AlbumSelection.compareTracks(b, a, TrackSort.DURATION) < 0)
        assertTrue(AlbumSelection.compareTracks(track(0, "Track 10"), track(1, "Track 2"), TrackSort.TITLE) > 0)
    }

    @Test fun albumSort() {
        val a = album(0, "B", year = "1999", n = 9uL)
        val b = album(1, "a", year = "2001", n = 2uL)
        assertTrue(AlbumSelection.compareAlbums(b, a, AlbumSort.NAME) < 0)
        assertTrue(AlbumSelection.compareAlbums(a, b, AlbumSort.YEAR) < 0)
        assertTrue(AlbumSelection.compareAlbums(b, a, AlbumSort.TRACK_COUNT) < 0)
    }

    @Test fun matchIgnoresCaseAndDiacritics() {
        val t = track(0, "Café Del Mar", artist = "Ünal")
        assertTrue(AlbumSelection.matches(t, "cafe"))
        assertTrue(AlbumSelection.matches(t, "UNAL"))
        assertTrue(AlbumSelection.matches(t, "al")) // album "Al"
        assertFalse(AlbumSelection.matches(t, "zzz"))
    }

    // SettingsState
    @Test fun settingsRoundTrip() {
        val f = File(tmp.root, "settings.json")
        val s = SettingsState(f)
        assertEquals(listOf("/a", "/b"), s.addFolders(listOf("/a/", "/b", "/a", " ")))
        s.animatedBackground = false
        s.backdropSource = BackdropSource.BLOCKS
        s.volume = 0.4f
        val r = SettingsState(f)
        assertEquals(listOf("/a", "/b"), r.musicFolders)
        assertFalse(r.animatedBackground)
        assertEquals(BackdropSource.BLOCKS, r.backdropSource)
        assertEquals(0.4f, r.volume, 0.001f)
        assertEquals("/a", r.removeFolder("/a/"))
        assertNull(r.removeFolder("/nope"))
        assertEquals(listOf("/b"), SettingsState(f).musicFolders)
    }

    @Test fun settingsUnknownAndMissingKeys() {
        val f = File(tmp.root, "settings.json")
        f.writeText("""{"backdropSource":"hologram","volume":7,"extra":1}""")
        val s = SettingsState(f)
        assertEquals(BackdropSource.ARTWORK, s.backdropSource)
        assertEquals(1f, s.volume, 0f)
        assertTrue(s.animatedBackground)
        assertTrue(s.musicFolders.isEmpty())
    }

    @Test fun settingsCorruptFileKeptAsBad() {
        val f = File(tmp.root, "settings.json")
        f.writeText("{not json")
        val s = SettingsState(f)
        assertTrue(s.animatedBackground)
        assertEquals("{not json", File(tmp.root, "settings.json.bad").readText())
        s.volume = 0.5f
        assertEquals(0.5f, SettingsState(f).volume, 0.001f)
    }

    // PlaybackClock math (engine-free)
    @Test fun clockMath() {
        val c = PlaybackClock(kotlinx.coroutines.CoroutineScope(kotlinx.coroutines.Dispatchers.Unconfined))
        assertNull(c.targetPositionMs(0.5f))
        c.adopt(uniffi.fluyer_core.PlayerBarViewModel(0, "t", "a", "b", 0uL, 200_000uL, 0f, "", true, NativeRepeatMode.NONE, false, 1f))
        assertEquals(100_000uL, c.targetPositionMs(0.5f))
        assertEquals(200_000uL, c.targetPositionMs(9f))
        assertEquals(0uL, c.targetPositionMs(-1f))
        c.applyLocalPosition(50_000uL)
        assertEquals(0.25f, c.progressPct, 0.0001f)
        assertEquals("0:50 / 3:20", c.timeLabel)
    }
}
