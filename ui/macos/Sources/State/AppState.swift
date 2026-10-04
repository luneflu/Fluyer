import Foundation
import FluyerCore
import Observation

/// Root of the app's observable state.
///
/// Ownership is split by concern so no single type accumulates engine, library,
/// playback and navigation duties:
///
/// - `playback`  — now-playing metadata, transport commands, position clock
/// - `library`   — scanned tracks and albums, scan progress
/// - `selection` — which album, if any, the grid is showing
/// - `toast`     — transient messages from the core
///
/// This type owns the engine, translates `FluyerEvent`s into updates on those
/// objects, and performs the cross-cutting full-library refresh that spans them.
@MainActor
@Observable
final class AppState: FluyerEventListener {
    let playback: PlaybackState
    let library: LibraryState
    let selection: AlbumSelection
    let toast: ToastState

    private(set) var engine: FluyerAppEngine?

    init() {
        let library = LibraryState()
        let playback = PlaybackState()
        let selection = AlbumSelection(library: library)
        self.library = library
        self.playback = playback
        self.selection = selection
        self.toast = ToastState()

        // Every stored property is initialized, so `self` can be handed to the core
        // as its event listener without risking a callback into a half-built state.
        let handle = EngineHandle()
        handle.attach(listener: self)
        engine = handle.engine
        playback.engine = handle.engine
        library.engine = handle.engine
        selection.engine = handle.engine

        refresh()
    }

    /// Re-read everything the window shows from the core.
    func refresh() {
        playback.reloadBar()
        playback.reloadPlayView()
        library.reload()
        selection.reload()
    }

    /// Ask the core to scan the folders the user picks.
    func promptAddFolder() {
        let paths = FolderPicker.musicFolders(message: "Choose music folders to add to Fluyer")
        guard !paths.isEmpty else { return }
        engine?.scanDirectories(directories: paths)
    }

    /// The core calls this from whichever thread the audio pipeline runs on, so hop
    /// to the main actor once and do all mutation there.
    nonisolated func onEvent(event: FluyerEvent) {
        Task { @MainActor [weak self] in
            self?.handle(event)
        }
    }

    private func handle(_ event: FluyerEvent) {
        switch event {
        case .playerBarUpdated:
            playback.reloadBar()

        case .playViewUpdated(let viewModel):
            // ponytail: declared by UniFFI but never emitted — `EventSink` in
            // `crates/fluyer_core/src/events.rs` has no play-view callback. Handled
            // anyway so the snapshot is used the moment core starts sending it.
            playback.applyPlayView(viewModel)

        case .trackChanged:
            playback.applyTrackChange()
            library.reloadActiveFlags()

        case .libraryUpdated:
            refresh()

        case .scanProgress(let status):
            library.scanStatus = status

        case .toast(let message):
            toast.show(message)

        case .trackCoverLoaded(let index):
            ThumbnailStore.shared.invalidate(prefix: ThumbnailKey.trackPrefix(index))
            playback.reloadPlayView()

        case .albumCoverLoaded(let index):
            ThumbnailStore.shared.invalidate(prefix: ThumbnailKey.albumPrefix(index))

        case .lyricsLoaded:
            playback.reloadPlayView()
        }
    }
}
