import XCTest
@testable import FluyerApp

final class TimeFormatTests: XCTestCase {
    func testElapsedTruncatesToSeconds() {
        XCTAssertEqual(TimeFormat.elapsed(0), "0:00")
        XCTAssertEqual(TimeFormat.elapsed(1_000), "0:01")
        XCTAssertEqual(TimeFormat.elapsed(59_999), "0:59")
        XCTAssertEqual(TimeFormat.elapsed(60_000), "1:00")
        XCTAssertEqual(TimeFormat.elapsed(3_599_000), "59:59")
        XCTAssertEqual(TimeFormat.elapsed(3_600_000), "60:00")
    }

    func testElapsedTruncatesPartialSeconds() {
        XCTAssertEqual(TimeFormat.elapsed(1_500), "0:01")
        XCTAssertEqual(TimeFormat.elapsed(199_999), "3:19")
    }

    /// The layout must match the core's `time_label`, which the player bar also
    /// receives verbatim from `PlayerBarUpdated`.
    func testPairMatchesCoreLayout() {
        XCTAssertEqual(TimeFormat.pair(0, 0), "0:00 / 0:00")
        XCTAssertEqual(TimeFormat.pair(61_000, 185_000), "1:01 / 3:05")
    }
}
