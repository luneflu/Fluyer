import FluyerCore
import Observation

/// What the library pane lists (`MusicListType` in the legacy webview).
///
/// ponytail: no `.playlist` / `.folder` yet; the core has no playlist or folder API.
enum LibraryMode: Hashable {
    /// Album carousel over the track grid.
    case tracks
    /// Full-height album grid.
    case albums
}

/// Track sort keys. `.album` is the core's library order (album, track number,
/// file name); inside an album that is track-number order.
enum TrackSort: String, CaseIterable, Identifiable {
    case album = "Album", title = "Title", artist = "Artist", duration = "Duration"
    var id: Self { self }
}

enum AlbumSort: String, CaseIterable, Identifiable {
    case name = "Name", artist = "Artist", year = "Year", trackCount = "Track Count"
    var id: Self { self }
}

/// Which album the track grid is showing, and the album-scoped playback commands.
///
/// Split out of the former monolithic `AppState`. The engine reference is assigned
/// once by `AppState` right after the core starts.
///
/// ponytail: `index` is the single source of truth and `detail` is derived from it.
/// The previous `AppState` kept the two as independent optionals and `displayedTracks`
/// had to check both, so a failed detail fetch silently desynced the selection.
@MainActor
@Observable
final class AlbumSelection {
    private let library: LibraryState

    private(set) var index: Int?
    private(set) var detail: AlbumDetailViewModel?

    var engine: FluyerAppEngine?

    init(library: LibraryState) {
        self.library = library
    }

    var isActive: Bool { index != nil }

    var mode: LibraryMode = .tracks

    /// Search text; matches title, artist or album (case- and diacritic-insensitive).
    var query = ""

    var trackSort = TrackSort.album
    var albumSort = AlbumSort.name
    /// Shared by both modes, like legacy `filterBarStore.sortAsc`.
    ///
    /// ponytail: sort is session-only; persist in `SettingsState` if users ask.
    var sortAscending = true

    /// Albums whose name or artist matches `query`.
    ///
    /// ponytail: legacy kept every album holding a matching track; name/artist only
    /// is enough until someone misses it, then match via `library.tracks`.
    var displayedAlbums: [AlbumCardViewModel] {
        let q = query.trimmingCharacters(in: .whitespacesAndNewlines)
        let albums = q.isEmpty ? library.albums : library.albums.filter { Self.matches([$0.name, $0.artist], q) }
        let sorted = albums.sorted { Self.ordered($0, $1, by: albumSort) }
        return sortAscending ? sorted : sorted.reversed()
    }

    /// Unfiltered rows: the album's tracks when one is selected, else the library.
    private var source: [TrackItemViewModel] {
        detail?.tracks ?? library.tracks
    }

    /// The rows the grid renders: `source` sorted, then narrowed by `query`.
    ///
    /// ponytail: re-sorted on every read; fine for a few thousand tracks. Cache on
    /// `source`/sort changes if big libraries stutter.
    var displayedTracks: [TrackItemViewModel] {
        let q = query.trimmingCharacters(in: .whitespacesAndNewlines)
        let tracks = sorted(source)
        return q.isEmpty ? tracks : tracks.filter { Self.matches($0, q) }
    }

    private func sorted(_ tracks: [TrackItemViewModel]) -> [TrackItemViewModel] {
        let ordered = trackSort == .album ? tracks : tracks.sorted { Self.ordered($0, $1, by: trackSort) }
        return sortAscending ? ordered : ordered.reversed()
    }

    // `index` breaks ties so equal keys keep library order.
    static func ordered(_ a: TrackItemViewModel, _ b: TrackItemViewModel, by sort: TrackSort) -> Bool {
        switch sort {
        case .album: return a.index < b.index
        case .title: return less(a.title, b.title, tie: a.index < b.index)
        case .artist: return less(a.artist, b.artist, tie: a.index < b.index)
        case .duration: return (a.durationMs, a.index) < (b.durationMs, b.index)
        }
    }

    static func ordered(_ a: AlbumCardViewModel, _ b: AlbumCardViewModel, by sort: AlbumSort) -> Bool {
        switch sort {
        case .name: return less(a.name, b.name, tie: a.index < b.index)
        case .artist: return less(a.artist, b.artist, tie: a.index < b.index)
        case .year: return less(a.year, b.year, tie: a.index < b.index)
        case .trackCount: return (a.trackCount, a.index) < (b.trackCount, b.index)
        }
    }

    /// Finder order: case-insensitive, numbers by value ("2" before "10").
    private static func less(_ a: String, _ b: String, tie: Bool) -> Bool {
        switch a.localizedStandardCompare(b) {
        case .orderedAscending: return true
        case .orderedDescending: return false
        case .orderedSame: return tie
        }
    }

    static func matches(_ track: TrackItemViewModel, _ query: String) -> Bool {
        matches([track.title, track.artist, track.album], query)
    }

    private static func matches(_ fields: [String], _ query: String) -> Bool {
        fields.contains { $0.range(of: query, options: [.caseInsensitive, .diacriticInsensitive]) != nil }
    }

    /// Play a displayed track. The queue is the sorted, unfiltered list, so it
    /// follows the grid's order and stays whole under an active search.
    func playTrack(_ track: TrackItemViewModel) {
        play(sorted(source), startingAt: track.path)
    }

    /// Hands the core library indices, resolved by path: album rows carry
    /// album-local indices.
    private func play(_ queue: [TrackItemViewModel], startingAt path: String?) {
        let byPath = Dictionary(library.tracks.map { ($0.path, $0.index) }, uniquingKeysWith: { first, _ in first })
        let indices = queue.compactMap { byPath[$0.path] }
        let start = path.flatMap { byPath[$0] }.flatMap { indices.firstIndex(of: $0) } ?? 0
        engine?.playLibraryTracks(indices: indices, startIndex: UInt64(start))
    }

    /// Open an album's tracks; from the album grid this returns to the track view.
    func select(_ index: Int) {
        self.index = index
        mode = .tracks
        detail = engine?.getAlbumDetail(index: UInt64(index))
    }

    func clear() {
        index = nil
        detail = nil
    }

    /// Re-read the selected album after a library rescan.
    func reload() {
        guard let index else {
            detail = nil
            return
        }
        detail = engine?.getAlbumDetail(index: UInt64(index))
    }

    // MARK: - Playback

    /// Header "Play": the open album in the grid's order.
    func playSelected() {
        guard let detail else { return }
        play(sorted(detail.tracks), startingAt: nil)
    }

    func queueSelected() { index.map(queueAlbum) }
    func shuffleSelected() { index.map(shuffleAlbum) }

    func playAlbum(_ index: Int) { engine?.playAlbum(index: UInt64(index)) }
    func queueAlbum(_ index: Int) { engine?.queueAlbum(index: UInt64(index)) }
    func shuffleAlbum(_ index: Int) { engine?.shuffleAlbum(index: UInt64(index)) }
}
