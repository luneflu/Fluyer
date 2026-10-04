import XCTest
import FluyerCore
@testable import FluyerApp

@MainActor
final class PlaybackStateTests: XCTestCase {
    // MARK: - playerBar composition

    /// Position lives in `clock`, never in `bar`. If the snapshot ever leaked through
    /// raw, these would read back as the placeholder's zeros and the progress bar
    /// would never move.
    func testPlayerBarReportsClockPositionNotSnapshotPosition() {
        let playback = PlaybackState()
        playback.clock.durationMs = 200_000
        playback.clock.applyLocalPosition(50_000)

        let bar = playback.playerBar

        XCTAssertEqual(bar.positionMs, 50_000)
        XCTAssertEqual(bar.durationMs, 200_000)
        XCTAssertEqual(bar.progressPct, 0.25)
        XCTAssertEqual(bar.timeLabel, "0:50 / 3:20")
    }

    func testApplyBarAdoptsMetadataTransportAndVolume() {
        let playback = PlaybackState()
        var snapshot = PlayerBarViewModel.noTrack
        snapshot.trackIndex = 4
        snapshot.title = "Weightless"
        snapshot.artist = "Marconi Union"
        snapshot.album = "Ambient Transmissions"
        snapshot.positionMs = 1_000
        snapshot.durationMs = 4_000
        snapshot.progressPct = 0.25
        snapshot.timeLabel = "0:01 / 0:04"
        snapshot.isPlaying = true
        snapshot.repeatMode = .all
        snapshot.isShuffled = true
        snapshot.volume = 0.4

        playback.applyBar(snapshot)

        let bar = playback.playerBar
        XCTAssertEqual(bar.trackIndex, 4)
        XCTAssertEqual(bar.title, "Weightless")
        XCTAssertEqual(bar.artist, "Marconi Union")
        XCTAssertEqual(bar.album, "Ambient Transmissions")
        XCTAssertTrue(bar.isPlaying)
        XCTAssertEqual(bar.repeatMode, .all)
        XCTAssertTrue(bar.isShuffled)
        XCTAssertEqual(bar.volume, 0.4)
        XCTAssertEqual(bar.positionMs, 1_000)
        XCTAssertEqual(bar.timeLabel, "0:01 / 0:04")
    }

    /// The core is authoritative for volume: `getPlayerBarView` reports the value the
    /// player stored, which is what replaces the old merge's hardcoded `1.0`.
    func testApplyBarOverridesLocalVolumeWithSnapshotVolume() {
        let playback = PlaybackState()
        playback.setVolume(0.25)

        var snapshot = PlayerBarViewModel.noTrack
        snapshot.volume = 0.4
        playback.applyBar(snapshot)

        XCTAssertEqual(playback.playerBar.volume, 0.4)
    }

    // MARK: - Volume

    func testSetVolumeClampsToUnity() {
        let playback = PlaybackState()

        playback.setVolume(3)
        XCTAssertEqual(playback.playerBar.volume, 1)

        playback.setVolume(-1)
        XCTAssertEqual(playback.playerBar.volume, 0)
    }

    func testToggleMuteRestoresLastAudibleVolume() {
        let playback = PlaybackState()
        playback.setVolume(0.6)

        playback.toggleMute()
        XCTAssertEqual(playback.playerBar.volume, 0)

        playback.toggleMute()
        XCTAssertEqual(playback.playerBar.volume, 0.6)
    }

    /// A fresh state starts at full volume, so the first `toggleMute` mutes; the second
    /// has to come back to something audible rather than to silence.
    func testToggleMuteRoundTripsFromTheInitialVolume() {
        let playback = PlaybackState()
        XCTAssertEqual(playback.playerBar.volume, 1.0)

        playback.toggleMute()
        XCTAssertEqual(playback.playerBar.volume, 0)

        playback.toggleMute()
        XCTAssertEqual(playback.playerBar.volume, 1.0)
    }

    func testUnmutingFromSilenceRestoresSomethingAudible() {
        let playback = PlaybackState()
        playback.setVolume(0)

        playback.toggleMute()

        XCTAssertGreaterThan(playback.playerBar.volume, 0)
    }

    // MARK: - Transport

    func testTogglePlayFlipsTransportOptimistically() {
        let playback = PlaybackState()
        XCTAssertFalse(playback.playerBar.isPlaying)

        playback.togglePlay()

        XCTAssertTrue(playback.playerBar.isPlaying)
    }

    func testSeekToFractionIsANoOpWithoutDuration() {
        let playback = PlaybackState()
        playback.clock.applyLocalPosition(1_234)

        playback.seek(toFraction: 0.5)

        XCTAssertEqual(playback.clock.positionMs, 1_234)
    }

    /// With no engine every command is inert rather than crashing — this is the state
    /// the window renders in if the core fails to start.
    func testCommandsAreInertWithoutAnEngine() {
        let playback = PlaybackState()

        playback.seek(toMs: 5_000)
        playback.seek(toFraction: .nan)
        playback.next()
        playback.previous()
        playback.cycleRepeat()
        playback.shuffle()

        XCTAssertEqual(playback.clock.positionMs, 0)
    }

    // MARK: - Now playing

    func testOpeningPlayViewEnablesLyricFollowing() {
        let playback = PlaybackState()
        XCTAssertFalse(playback.clock.followsLyrics)

        playback.showPlayView = true
        XCTAssertTrue(playback.clock.followsLyrics)

        playback.showPlayView = false
        XCTAssertFalse(playback.clock.followsLyrics)
    }

    func testApplyPlayViewAdoptsSnapshot() {
        let playback = PlaybackState()
        var viewModel = PlayViewModel.empty
        viewModel.lyrics = [
            LyricLine(timestampMs: 0, text: "first"),
            LyricLine(timestampMs: 1_000, text: "second")
        ]

        playback.applyPlayView(viewModel)

        XCTAssertEqual(playback.playView.lyrics.count, 2)
    }
}
