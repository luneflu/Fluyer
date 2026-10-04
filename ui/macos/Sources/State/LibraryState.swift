import FluyerCore
import Observation

/// The scanned library: the flat track list, the album list and scan progress.
///
/// Split out of the former monolithic `AppState`. The engine reference is assigned
/// once by `AppState` right after the core starts.
@MainActor
@Observable
final class LibraryState {
    var tracks: [TrackItemViewModel] = []
    var albums: [AlbumCardViewModel] = []
    var scanStatus = ScanStatusViewModel.idle

    var engine: FluyerAppEngine?

    /// Re-read the whole library from the core.
    ///
    /// ponytail: `compactMap` over a range is enough here; an earlier revision spelled
    /// the same loop out by hand with a `reserveCapacity` that bought nothing, since
    /// `compactMap` sizes its result from the source.
    func reload() {
        guard let engine else { return }
        scanStatus = engine.getScanStatus()
        tracks = (0..<Int(engine.getTrackCount())).compactMap { engine.getTrackView(index: UInt64($0)) }
        albums = (0..<Int(engine.getAlbumCount())).compactMap { engine.getAlbumCard(index: UInt64($0)) }
    }

    /// Refresh only the "now playing" marker, so a track change does not rebuild
    /// every row in the grid.
    func reloadActiveFlags() {
        guard let engine, tracks.count == Int(engine.getTrackCount()) else { return }
        for row in tracks.indices {
            if let updated = engine.getTrackView(index: UInt64(row)) {
                tracks[row].isCurrent = updated.isCurrent
            }
        }
    }
}
