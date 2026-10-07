import FluyerCore
import Observation

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

    /// Search text; matches title, artist or album (case- and diacritic-insensitive).
    var query = ""

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
        [track.title, track.artist, track.album].contains {
            $0.range(of: query, options: [.caseInsensitive, .diacriticInsensitive]) != nil
        }
    }

    /// Play a displayed track; resolves its row in the unfiltered list so the queue
    /// stays whole under an active search.
    func playTrack(_ track: TrackItemViewModel) {
        guard let row = source.firstIndex(where: { $0.path == track.path }) else { return }
        playTrack(atRow: row)
    }

    func select(_ index: Int) {
        self.index = index
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

    func playSelected() {
        guard let index else { return }
        engine?.playAlbum(index: UInt64(index))
    }

    func queueSelected() {
        guard let index else { return }
        engine?.queueAlbum(index: UInt64(index))
    }

    func shuffleSelected() {
        guard let index else { return }
        engine?.shuffleAlbum(index: UInt64(index))
    }
}
