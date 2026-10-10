package org.alvindimas05.fluyer.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import uniffi.fluyer_core.AlbumCardViewModel
import uniffi.fluyer_core.AlbumDetailViewModel
import uniffi.fluyer_core.FluyerAppEngine
import uniffi.fluyer_core.TrackItemViewModel
import java.text.Normalizer

/** What the library pane lists. ponytail: no playlist / folder modes; the core has no such API. */
enum class LibraryMode { TRACKS, ALBUMS }

/**
 * Track sort keys. ALBUM is the core's library order (album, track number, file name);
 * inside an album that is track-number order.
 */
enum class TrackSort(val label: String) { ALBUM("Album"), TITLE("Title"), ARTIST("Artist"), DURATION("Duration") }

enum class AlbumSort(val label: String) { NAME("Name"), ARTIST("Artist"), YEAR("Year"), TRACK_COUNT("Track Count") }

/**
 * Which album the track list is showing, and the album-scoped playback commands.
 *
 * ponytail: [index] is the single source of truth and [detail] is derived from it, so a
 * failed detail fetch cannot desync the selection.
 */
class AlbumSelection(private val library: LibraryState) {
    var index by mutableStateOf<Int?>(null); private set
    var detail by mutableStateOf<AlbumDetailViewModel?>(null); private set

    var engine: FluyerAppEngine? = null

    val isActive get() = index != null

    var mode by mutableStateOf(LibraryMode.TRACKS)

    /** Search text; matches title, artist or album (case- and diacritic-insensitive). */
    var query by mutableStateOf("")

    var trackSort by mutableStateOf(TrackSort.ALBUM)
    var albumSort by mutableStateOf(AlbumSort.NAME)

    /** Shared by both modes. ponytail: session-only; persist in SettingsState if users ask. */
    var sortAscending by mutableStateOf(true)

    /**
     * Albums whose name or artist matches [query].
     * ponytail: name/artist only; match via library.tracks if someone misses album-by-track hits.
     */
    val displayedAlbums: List<AlbumCardViewModel>
        get() {
            val q = query.trim()
            val albums = if (q.isEmpty()) library.albums else library.albums.filter { matches(listOf(it.name, it.artist), q) }
            val sorted = albums.sortedWith { a, b -> compareAlbums(a, b, albumSort) }
            return if (sortAscending) sorted else sorted.reversed()
        }

    /** Unfiltered rows: the album's tracks when one is selected, else the library. */
    private val source get() = detail?.tracks ?: library.tracks

    /**
     * The rows the list renders: [source] sorted, then narrowed by [query].
     * ponytail: re-sorted on every read; fine for a few thousand tracks.
     */
    val displayedTracks: List<TrackItemViewModel>
        get() {
            val q = query.trim()
            val tracks = sorted(source)
            return if (q.isEmpty()) tracks else tracks.filter { matches(it, q) }
        }

    private fun sorted(tracks: List<TrackItemViewModel>): List<TrackItemViewModel> {
        val ordered = if (trackSort == TrackSort.ALBUM) tracks else tracks.sortedWith { a, b -> compareTracks(a, b, trackSort) }
        return if (sortAscending) ordered else ordered.reversed()
    }

    /** Play a displayed track. The queue is the sorted, unfiltered list, so it stays whole under a search. */
    fun playTrack(track: TrackItemViewModel) = play(sorted(source), track.path)

    /** Hands the core library indices, resolved by path: album rows carry album-local indices. */
    private fun play(queue: List<TrackItemViewModel>, startingAt: String?) {
        val byPath = HashMap<String, ULong>()
        for (t in library.tracks) byPath.putIfAbsent(t.path, t.index)
        val indices = queue.mapNotNull { byPath[it.path] }
        val start = startingAt?.let { byPath[it] }?.let { indices.indexOf(it) }?.takeIf { it >= 0 } ?: 0
        engine?.libraryPlayTracks(indices, start.toULong())
    }

    /** Open an album's tracks; from the album grid this returns to the track view. */
    fun select(index: Int) {
        this.index = index
        mode = LibraryMode.TRACKS
        detail = engine?.albumGetDetail(index.toULong())
    }

    fun clear() {
        index = null
        detail = null
    }

    /** Re-read the selected album after a library rescan. */
    fun reload() {
        detail = index?.let { engine?.albumGetDetail(it.toULong()) }
    }

    // Playback

    /** Header "Play": the open album in the list's order. */
    fun playSelected() {
        val d = detail ?: return
        play(sorted(d.tracks), null)
    }

    fun queueSelected() { index?.let(::queueAlbum) }
    fun shuffleSelected() { index?.let(::shuffleAlbum) }
    fun playAlbum(index: Int) { engine?.albumPlay(index.toULong()) }
    fun queueAlbum(index: Int) { engine?.albumQueue(index.toULong()) }
    fun shuffleAlbum(index: Int) { engine?.albumShuffle(index.toULong()) }

    companion object {
        // `index` breaks ties so equal keys keep library order.
        fun compareTracks(a: TrackItemViewModel, b: TrackItemViewModel, by: TrackSort): Int {
            val c = when (by) {
                TrackSort.ALBUM -> 0
                TrackSort.TITLE -> naturalCompare(a.title, b.title)
                TrackSort.ARTIST -> naturalCompare(a.artist, b.artist)
                TrackSort.DURATION -> a.durationMs.compareTo(b.durationMs)
            }
            return if (c != 0) c else a.index.compareTo(b.index)
        }

        fun compareAlbums(a: AlbumCardViewModel, b: AlbumCardViewModel, by: AlbumSort): Int {
            val c = when (by) {
                AlbumSort.NAME -> naturalCompare(a.name, b.name)
                AlbumSort.ARTIST -> naturalCompare(a.artist, b.artist)
                AlbumSort.YEAR -> naturalCompare(a.year, b.year)
                AlbumSort.TRACK_COUNT -> a.trackCount.compareTo(b.trackCount)
            }
            return if (c != 0) c else a.index.compareTo(b.index)
        }

        fun matches(track: TrackItemViewModel, query: String) = matches(listOf(track.title, track.artist, track.album), query)

        fun matches(fields: List<String>, query: String): Boolean {
            val q = fold(query)
            return fields.any { fold(it).contains(q) }
        }

        private fun fold(s: String) =
            Normalizer.normalize(s, Normalizer.Form.NFD).replace(Regex("\\p{M}+"), "").lowercase()
    }
}
