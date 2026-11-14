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

    // SettingsState: the core owns the file and normalization (services/settings.rs).
    @Test fun settingsDefaults() {
        val s = SettingsState()
        assertTrue(s.musicFolders.isEmpty())
        assertTrue(s.animatedBackground)
        assertEquals(BackdropSource.ARTWORK, s.backdropSource)
        assertEquals(1f, s.volume, 0f)
    }

    @Test fun settingsRemembersFoldersWithoutAnEngine() {
        val s = SettingsState()
        // Without an engine nothing canonicalizes: dedupe/trailing-slash rules live in the core.
        assertEquals(listOf("/a/", "/b"), s.addFolders(listOf("/a/", "/b")))
        assertEquals(listOf("/a/", "/b"), s.musicFolders)
    }

    @Test fun settingsRemoveFolderMatchesTrailingSeparator() {
        val s = SettingsState()
        s.addFolders(listOf("/a", "/b"))
        assertEquals("/a", s.removeFolder("/a/"))
        assertNull(s.removeFolder("/a"))
        assertEquals("/", SettingsState().apply { addFolders(listOf("/")) }.removeFolder("//"))
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
