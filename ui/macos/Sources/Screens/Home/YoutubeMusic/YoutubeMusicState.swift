import FluyerCore
import Observation

/// YouTube Music home feed, anonymous for now: shelves from the core, and playing a
/// track through it (it resolves the stream URL and feeds BASS).
@MainActor
@Observable
final class YoutubeMusicState {
    /// Assigned by `AppState` right after the core starts.
    @ObservationIgnored var engine: FluyerAppEngine?

    /// Home feed shelves, loaded once per app run.
    private(set) var shelves: [FeedShelf] = []
    private(set) var isLoadingHome = false
    private(set) var isLoadingMore = false
    /// Next-page token; `nil` means the feed is exhausted.
    private(set) var continuation: String?
    private(set) var error: String?
    /// Track currently handed to the player, so its row can show a speaker icon.
    private(set) var playingVideoId: String?

    /// Home feed for the anonymous account. Cached in `shelves`, so re-entering the
    /// tab does not refetch.
    func loadHome() async {
        guard let engine, shelves.isEmpty, !isLoadingHome else { return }
        isLoadingHome = true
        error = nil
        do {
            let feed = try await engine.providerHome()
            shelves = feed.shelves
            continuation = feed.continuation
        } catch {
            self.error = "YouTube Music: \(error.localizedDescription)"
        }
        isLoadingHome = false
    }

    /// Appends the next home page; called when the last shelf scrolls into view.
    func loadMore() async {
        guard let engine, let token = continuation, !isLoadingMore, !isLoadingHome else { return }
        isLoadingMore = true
        defer { isLoadingMore = false }
        do {
            let feed = try await engine.providerHomeMore(continuation: token)
            shelves += feed.shelves
            continuation = feed.continuation
        } catch {
            self.error = "YouTube Music: \(error.localizedDescription)"
        }
    }

    /// Feed items are flat, so the row knows everything the player needs.
    func play(_ item: SearchItem) async {
        guard let engine, let videoId = item.videoId else { return }
        let (artist, album) = Self.credit(item.subtitle)
        do {
            try await engine.providerPlayTrack(videoId: videoId, title: item.title, artist: artist,
                                               album: album, thumbnail: item.thumbnail, durationMs: 0)
            playingVideoId = videoId
        } catch {
            self.error = "Playback failed: \(error.localizedDescription)"
        }
    }

    /// Feed subtitles are "Artist • 3.6B plays" or "Artist • Album • 2021", so the stats
    /// segment is dropped before the first two real fields are taken as artist/album.
    private static func credit(_ subtitle: String?) -> (artist: String, album: String) {
        let fields = (subtitle ?? "")
            .components(separatedBy: "•")
            .map { $0.trimmingCharacters(in: .whitespaces) }
            .filter { !$0.isEmpty && !$0.contains("play") && !$0.contains("view") }
        return (fields.first ?? "", fields.count > 1 ? fields[1] : "")
    }
}
