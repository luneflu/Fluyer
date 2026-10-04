import FluyerCore
import Foundation
import Observation

/// Locally-ticked playback position and the active lyric cursor.
///
/// ponytail: the core only emits `PlayerBarUpdated` on discrete transport actions,
/// never on a timer, so continuous progress has to be sampled on the UI side. This
/// state is deliberately split out of `PlaybackState.bar` because it changes ~4x a
/// second: keeping it separate means a tick invalidates only the widgets that read
/// position, instead of every view bound to the player bar.
@MainActor
@Observable
final class PlaybackClock {
    /// Samples per second. Each tick is one FFI call plus SwiftUI layout.
    private static let tickInterval: TimeInterval = 0.25

    var positionMs: UInt64 = 0
    var durationMs: UInt64 = 0
    var progressPct: Float = 0
    var timeLabel: String = TimeFormat.pair(0, 0)

    /// Index into `PlayViewModel.lyrics`. Owned here rather than read off the core's
    /// `play_view` snapshot because it advances at tick rate, and refreshing
    /// `play_view` on every tick would rebuild the lyrics array 4x a second.
    var currentLyricIndex: Int = -1

    /// Set once by `PlaybackState`; nil means there is nothing to sample.
    private weak var engine: FluyerAppEngine?

    /// Whether to advance `currentLyricIndex`. Only the now-playing screen needs it.
    var followsLyrics = false

    @ObservationIgnored private var timer: Timer?

    func start(sampling engine: FluyerAppEngine?) {
        self.engine = engine
        guard timer == nil, engine != nil else { return }
        let timer = Timer(timeInterval: Self.tickInterval, repeats: true) { [weak self] _ in
            Task { @MainActor in
                self?.tick()
            }
        }
        // `.default` alone stops firing while a menu or window drag holds the run
        // loop, which would freeze the progress bar mid-track.
        RunLoop.main.add(timer, forMode: .common)
        self.timer = timer
    }

    func stop() {
        timer?.invalidate()
        timer = nil
    }

    /// Adopt the position fields of an authoritative core snapshot verbatim.
    func adopt(_ snapshot: PlayerBarViewModel) {
        positionMs = snapshot.positionMs
        durationMs = snapshot.durationMs
        progressPct = snapshot.progressPct
        timeLabel = snapshot.timeLabel
    }

    /// Recompute the derived position fields after a local seek, before the core has
    /// echoed anything back. Mirrors the core's own clamping in `lib.rs`.
    func applyLocalPosition(_ positionMs: UInt64) {
        self.positionMs = positionMs
        guard durationMs > 0 else { return }
        progressPct = Float(positionMs) / Float(durationMs)
        timeLabel = TimeFormat.pair(positionMs, durationMs)
    }

    /// Position targeted by a drag on the progress bar, or `nil` when there is no
    /// duration to seek within.
    func targetPositionMs(forFraction fraction: Float) -> UInt64? {
        guard durationMs > 0 else { return nil }
        let clamped = fraction.clamped(to: 0...1)
        return UInt64(Float(durationMs) * clamped)
    }

    private func tick() {
        guard let engine else { return }
        let position = engine.getPosition()
        positionMs = position
        guard durationMs > 0 else { return }
        progressPct = Float(position) / Float(durationMs)
        timeLabel = TimeFormat.pair(position, durationMs)
        if followsLyrics {
            currentLyricIndex = Int(engine.getActiveLyricIndex(positionMs: position))
        }
    }
}
