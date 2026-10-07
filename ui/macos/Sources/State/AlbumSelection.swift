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

    /// Albums whose name or artist matches `query`.
    ///
    /// ponytail: legacy kept every album holding a matching track; name/artist only
    /// is enough until someone misses it, then match via `library.tracks`.
    var displayedAlbums: [AlbumCardViewModel] {
        let q = query.trimmingCharacters(in: .whitespacesAndNewlines)
        return q.isEmpty ? library.albums : library.albums.filter { Self.matches([$0.name, $0.artist], q) }
    }

    /// Unfiltered rows: the album's tracks when one is selected, else the library.
    private var source: [TrackItemViewModel] {
        detail?.tracks ?? library.tracks
    }

    /// The rows the grid renders: `source` narrowed by `query`.
    var displayedTracks: [TrackItemViewModel] {
        let q = query.trimmingCharacters(in: .whitespacesAndNewlines)
        return q.isEmpty ? source : source.filter { Self.matches($0, q) }
    }

    static func matches(_ track: TrackItemViewModel, _ query: String) -> Bool {
        matches([track.title, track.artist, track.album], query)
    }

    private static func matches(_ fields: [String], _ query: String) -> Bool {
        fields.contains { $0.range(of: query, options: [.caseInsensitive, .diacriticInsensitive]) != nil }
    }

    /// Play a displayed track; resolves its row in the unfiltered list so the queue
    /// stays whole under an active search.
    func playTrack(_ track: TrackItemViewModel) {
        guard let row = source.firstIndex(where: { $0.path == track.path }) else { return }
        playTrack(atRow: row)
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

    /// Play a row of `displayedTracks`, resolving it against the selected album.
    func playTrack(atRow row: Int) {
        guard row >= 0 else { return }
        if let index {
            engine?.playAlbumTrack(albumIndex: UInt64(index), trackIndex: UInt64(row))
        } else {
            engine?.playAllFromLibrary(startIndex: UInt64(row))
        }
    }

    func playSelected() { index.map(playAlbum) }
    func queueSelected() { index.map(queueAlbum) }
    func shuffleSelected() { index.map(shuffleAlbum) }

    func playAlbum(_ index: Int) { engine?.playAlbum(index: UInt64(index)) }
    func queueAlbum(_ index: Int) { engine?.queueAlbum(index: UInt64(index)) }
    func shuffleAlbum(_ index: Int) { engine?.shuffleAlbum(index: UInt64(index)) }
}
