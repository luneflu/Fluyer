import FluyerCore
import Observation

/// Now-playing state: metadata, transport flags, the position clock and every
/// command the UI issues to the player.
///
/// Split out of the former monolithic `AppState`. The engine reference is assigned
/// once by `AppState` right after the core starts.
@MainActor
@Observable
final class PlaybackState {
    /// Metadata and transport flags. Position is deliberately *not* read from here —
    /// `clock` owns it, and `playerBar` overlays the clock onto this snapshot.
    private(set) var bar = PlayerBarViewModel.noTrack

    /// Core's now-playing snapshot. Its `currentLyricIndex` is unused; see
    /// `PlaybackClock.currentLyricIndex`.
    private(set) var playView = PlayViewModel.empty

    /// Whether the full-screen now-playing surface is presented.
    var showPlayView = false {
        didSet {
            guard showPlayView != oldValue else { return }
            clock.followsLyrics = showPlayView
            if showPlayView { syncLyricCursor() }
        }
    }

    let clock = PlaybackClock()

    /// Remembers the last audible volume so unmuting restores it.
    private var lastAudibleVolume: Float = 1.0

    private static let audibleThreshold: Float = 0.001
    private static let minimumRestoreVolume: Float = 0.05

    var engine: FluyerAppEngine?

    // MARK: - Snapshots

    /// The core's player-bar view model with the locally-ticked position filled in.
    var playerBar: PlayerBarViewModel {
        var snapshot = bar
        snapshot.positionMs = clock.positionMs
        snapshot.durationMs = clock.durationMs
        snapshot.progressPct = clock.progressPct
        snapshot.timeLabel = clock.timeLabel
        return snapshot
    }

    // MARK: - Core refreshes

    /// Adopt an authoritative player-bar snapshot from the core.
    ///
    /// ponytail: `FluyerEvent::PlayerBarUpdated` looks like it carries the full view
    /// model but it does not — the Rust bridge fills `title`/`artist`/`album` with
    /// empty strings and hardcodes `volume` to 1.0
    /// (`crates/fluyer_core/src/uniffi_api.rs`). The previous handler worked around
    /// that by re-reading `getPlayerBarView()` on every tick and sniffing the title
    /// against `"No Track"` to decide which fields to trust. The snapshot from
    /// `getPlayerBarView()` is complete and authoritative — including the real volume
    /// — so take it whole and drop the second call's field-by-field merge.
    func applyBar(_ snapshot: PlayerBarViewModel) {
        bar = snapshot
        clock.adopt(snapshot)
        if showPlayView { reloadPlayView() }
        syncClockToTransport()
    }

    func reloadBar() {
        guard let engine else { return }
        applyBar(engine.getPlayerBarView())
    }

    func reloadPlayView() {
        guard let engine else { return }
        playView = engine.getPlayView()
    }

    /// Adopt a now-playing snapshot the core already sent us.
    func applyPlayView(_ viewModel: PlayViewModel) {
        playView = viewModel
    }

    /// Fold a track change into the now-playing state.
    ///
    /// ponytail: the old handler had an engine-less fallback that copied the event's
    /// track into the player bar. That branch was unreachable — `TrackChanged` can only
    /// be emitted by an engine — so it is gone rather than ported.
    func applyTrackChange() {
        guard let engine else { return }
        reloadBar()
        reloadPlayView()
        syncLyricCursor()
    }

    // MARK: - Transport commands

    func togglePlay() {
        engine?.togglePlay()
        // Optimistic: the core confirms with its own `PlayerBarUpdated`.
        bar.isPlaying.toggle()
        syncClockToTransport()
    }

    func next() { engine?.next() }

    func previous() { engine?.previous() }

    func cycleRepeat() { engine?.cycleRepeat() }

    func shuffle() { engine?.shuffle() }

    func seek(toMs positionMs: UInt64) {
        guard let engine else { return }
        engine.seek(positionMs: positionMs)
        clock.applyLocalPosition(positionMs)
        syncLyricCursor()
    }

    /// Seek from a fraction of the track, as produced by dragging the progress bar.
    func seek(toFraction fraction: Float) {
        guard let target = clock.targetPositionMs(forFraction: fraction) else { return }
        seek(toMs: target)
    }

    func setVolume(_ volume: Float) {
        let clamped = volume.clamped(to: 0...1)
        if clamped > Self.audibleThreshold {
            lastAudibleVolume = clamped
        }
        bar.volume = clamped
        engine?.setVolume(volume: clamped)
    }

    func toggleMute() {
        if bar.volume > Self.audibleThreshold {
            setVolume(0)
        } else {
            setVolume(max(lastAudibleVolume, Self.minimumRestoreVolume))
        }
    }

    // MARK: - Clock

    private func syncClockToTransport() {
        if bar.isPlaying {
            clock.start(sampling: engine)
        } else {
            clock.stop()
        }
    }

    private func syncLyricCursor() {
        guard showPlayView, let engine else { return }
        clock.currentLyricIndex = Int(engine.getActiveLyricIndex(positionMs: clock.positionMs))
    }
}
