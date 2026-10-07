import XCTest
@testable import FluyerApp

@MainActor
final class QueueStateTests: XCTestCase {
    /// Queue [A, B, C, D]. `onMove` offsets are gaps before removal.
    func testDropOffsetMapsToFinalIndex() {
        XCTAssertEqual(QueueState.target(from: 0, dropOffset: 4), 3, "A to end")
        XCTAssertEqual(QueueState.target(from: 0, dropOffset: 2), 1, "A between B and C")
        XCTAssertEqual(QueueState.target(from: 3, dropOffset: 0), 0, "D to top")
        XCTAssertEqual(QueueState.target(from: 3, dropOffset: 1), 1, "D between A and B")
        XCTAssertEqual(QueueState.target(from: 1, dropOffset: 1), 1, "drop on own top gap")
        XCTAssertEqual(QueueState.target(from: 1, dropOffset: 2), 1, "drop on own bottom gap")
    }
}
