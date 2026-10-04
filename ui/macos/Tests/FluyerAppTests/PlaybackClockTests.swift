import XCTest
import FluyerCore
@testable import FluyerApp

@MainActor
final class PlaybackClockTests: XCTestCase {
    func testApplyLocalPositionRecomputesDerivedFields() {
        let clock = PlaybackClock()
        clock.durationMs = 200_000

        clock.applyLocalPosition(50_000)

        XCTAssertEqual(clock.positionMs, 50_000)
        XCTAssertEqual(clock.progressPct, 0.25)
        XCTAssertEqual(clock.timeLabel, "0:50 / 3:20")
    }

    func testApplyLocalPositionWithoutDurationLeavesProgressAtZero() {
        let clock = PlaybackClock()
        clock.durationMs = 0

        clock.applyLocalPosition(50_000)

        XCTAssertEqual(clock.positionMs, 50_000)
        XCTAssertEqual(clock.progressPct, 0)
    }

    /// `adopt` takes the core's own pre-formatted values verbatim so a tick never
    /// disagrees with the `time_label` the core shipped.
    func testAdoptTakesSnapshotFieldsVerbatim() {
        let clock = PlaybackClock()
        var snapshot = PlayerBarViewModel.noTrack
        snapshot.positionMs = 12_345
        snapshot.durationMs = 60_000
        snapshot.progressPct = 0.2
        snapshot.timeLabel = "0:12 / 1:00"

        clock.adopt(snapshot)

        XCTAssertEqual(clock.positionMs, 12_345)
        XCTAssertEqual(clock.durationMs, 60_000)
        XCTAssertEqual(clock.progressPct, 0.2)
        XCTAssertEqual(clock.timeLabel, "0:12 / 1:00")
    }

    func testTargetPositionClampsFraction() {
        let clock = PlaybackClock()
        clock.durationMs = 1_000

        XCTAssertEqual(clock.targetPositionMs(forFraction: 0.5), 500)
        XCTAssertEqual(clock.targetPositionMs(forFraction: -2), 0)
        XCTAssertEqual(clock.targetPositionMs(forFraction: 4), 1_000)
    }

    func testTargetPositionIsNilWithoutDuration() {
        let clock = PlaybackClock()
        clock.durationMs = 0
        XCTAssertNil(clock.targetPositionMs(forFraction: 0.5))
    }

    /// Regression: a drag inside a zero-width frame yields `0 / 0` == NaN, and
    /// `UInt64(Float.nan)` traps.
    func testTargetPositionSurvivesNonFiniteFraction() {
        let clock = PlaybackClock()
        clock.durationMs = 1_000
        XCTAssertEqual(clock.targetPositionMs(forFraction: .nan), 0)
        XCTAssertEqual(clock.targetPositionMs(forFraction: .infinity), 1_000)
    }
}
