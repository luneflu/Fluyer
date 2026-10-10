import AppKit
import FluyerCore
import MediaPlayer

/// Bridges macOS Now Playing (Control Center widget, media keys, AirPods/Touch Bar
/// controls) to `PlaybackState`. macOS twin of Windows' `MediaTransportCoordinator`.
@MainActor
final class NowPlayingCoordinator {
    private let state: AppState
    private var trackKey = ""
    private var artwork: MPMediaItemArtwork?

    init(state: AppState) {
        self.state = state
        let center = MPRemoteCommandCenter.shared()
        bind(center.playCommand) { $0.playback.play() }
        bind(center.pauseCommand) { $0.playback.pause() }
        bind(center.stopCommand) { $0.playback.pause() }
        bind(center.togglePlayPauseCommand) { $0.playback.togglePlay() }
        bind(center.nextTrackCommand) { $0.playback.next() }
        bind(center.previousTrackCommand) { $0.playback.previous() }
        bind(center.changeRepeatModeCommand) { $0.playback.cycleRepeat() }
        bind(center.changeShuffleModeCommand) { $0.playback.shuffle() }
        center.changePlaybackPositionCommand.addTarget { [weak self] event in
            guard let event = event as? MPChangePlaybackPositionCommandEvent else { return .commandFailed }
            let ms = UInt64(max(0, event.positionTime * 1000))
            MainActor.assumeIsolated { self?.state.playback.seek(toMs: ms) }
            return .success
        }
        observe()
    }

    private func bind(_ command: MPRemoteCommand, _ action: @escaping @MainActor (AppState) -> Void) {
        command.addTarget { [weak self] _ in
            // MediaPlayer delivers remote commands on the main thread.
            MainActor.assumeIsolated {
                guard let self else { return }
                action(self.state)
            }
            return .success
        }
    }

    /// Re-arms itself after every change to the bar snapshot.
    private func observe() {
        withObservationTracking {
            update(state.playback.bar)
        } onChange: { [weak self] in
            Task { @MainActor in self?.observe() }
        }
    }

    private func update(_ bar: PlayerBarViewModel) {
        let info = MPNowPlayingInfoCenter.default()
        let hasTrack = bar.trackIndex >= 0 && !bar.title.isEmpty && bar.title != PlayerBarViewModel.noTrack.title
        guard hasTrack else {
            trackKey = ""
            artwork = nil
            info.nowPlayingInfo = nil
            info.playbackState = .stopped
            return
        }

        let key = "\(bar.trackIndex)|\(bar.title)|\(bar.artist)|\(bar.album)"
        if key != trackKey {
            trackKey = key
            artwork = nil
            Task { await loadArtwork(for: key) }
        }

        var now: [String: Any] = [
            MPMediaItemPropertyTitle: bar.title,
            MPMediaItemPropertyArtist: bar.artist,
            MPMediaItemPropertyAlbumTitle: bar.album,
            MPMediaItemPropertyPlaybackDuration: Double(bar.durationMs) / 1000,
            MPNowPlayingInfoPropertyElapsedPlaybackTime: Double(bar.positionMs) / 1000,
            MPNowPlayingInfoPropertyPlaybackRate: bar.isPlaying ? 1.0 : 0.0,
            MPNowPlayingInfoPropertyMediaType: MPNowPlayingInfoMediaType.audio.rawValue,
        ]
        if let artwork { now[MPMediaItemPropertyArtwork] = artwork }
        info.nowPlayingInfo = now
        info.playbackState = bar.isPlaying ? .playing : .paused
    }

    private func loadArtwork(for key: String) async {
        guard let image = await state.covers.currentUncached(px: 400),
              key == trackKey else { return }
        artwork = MPMediaItemArtwork(boundsSize: image.size) { _ in image }
        update(state.playback.bar)
    }
}
