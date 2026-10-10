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
/// - `queue`     — play-queue snapshot for the queue sidebar
/// - `settings`  — persisted user settings (folders, toggles, volume)
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
    let queue: QueueState
    let settings: SettingsState

    private(set) var engine: FluyerAppEngine?

    /// - Parameter settings: defaults to `settings.json` next to the core's data.
    init(settings: SettingsState? = nil) {
        let library = LibraryState()
        let playback = PlaybackState()
        let selection = AlbumSelection(library: library)
        self.library = library
        self.playback = playback
        self.selection = selection
        self.toast = ToastState()
        self.queue = QueueState()
        self.settings = settings ?? SettingsState(
            url: EngineHandle.dataDirectory.appendingPathComponent(SettingsState.fileName))

        // Every stored property is initialized, so `self` can be handed to the core
        // as its event listener without risking a callback into a half-built state.
        let handle = EngineHandle()
        handle.attach(listener: self)
        attach(handle.engine)
    }

    /// Bind the engine to every state object, then restore persisted settings and
    /// re-scan saved folders so files changed while closed show up (unchanged files
    /// are skipped by mtime).
    func attach(_ engine: FluyerAppEngine?) {
        self.engine = engine
        playback.engine = engine
        library.engine = engine
        selection.engine = engine
        queue.engine = engine

        refresh()
        guard let engine else { return }
        engine.discordSetEnabled(enabled: settings.discordRpc)
        playback.setVolume(settings.volume)
        scanSavedFolders()
    }

    /// Re-read everything the window shows from the core.
    func refresh() {
        playback.reloadBar()
        playback.reloadPlayView()
        library.reload()
        selection.reload()
        queue.reload()
    }

    /// Ask for folders, remember them and scan them.
    func promptAddFolder() {
        scanFolders(FolderPicker.musicFolders(message: "Choose music folders to add to Fluyer"))
    }

    func scanFolders(_ paths: [String]) {
        guard !paths.isEmpty else { return }
        settings.addFolders(paths)
        engine?.libraryScan(directories: paths.map(SettingsState.normalize))
    }

    /// Re-scan every saved folder.
    func scanSavedFolders() {
        guard !settings.musicFolders.isEmpty else { return }
        engine?.libraryScan(directories: settings.musicFolders)
    }

    /// Forget a folder and drop its tracks from the library.
    func removeFolder(_ path: String) {
        guard let stored = settings.removeFolder(path) else { return }
        engine?.libraryRemoveFolder(directory: stored)
        refresh()
    }

    func setDiscordEnabled(_ enabled: Bool) {
        settings.discordRpc = enabled
        engine?.discordSetEnabled(enabled: enabled)
    }

    /// Play the whole library from the first track (legacy menu "Play All").
    func playAll() {
        guard !library.tracks.isEmpty else { return }
        engine?.libraryPlayAll(startIndex: 0)
    }

    /// Persist session state that changes too often to save live (volume).
    func saveSession() {
        settings.volume = playback.bar.volume
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
            queue.reload()

        case .playViewUpdated(let viewModel):
            // ponytail: declared by UniFFI but never emitted — `EventSink` in
            // `crates/fluyer_core/src/events.rs` has no play-view callback. Handled
            // anyway so the snapshot is used the moment core starts sending it.
            playback.applyPlayView(viewModel)

        case .trackChanged:
            playback.applyTrackChange()
            library.reloadActiveFlags()
            queue.reload()

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
