import XCTest
@testable import FluyerApp

final class QueueEdgeTriggerTests: XCTestCase {
    private let t = QueueEdgeTrigger(size: CGSize(width: 1000, height: 600), panelMinX: 600)

    func testClosedOpensOnlyOnTheEdge() {
        XCTAssertEqual(t.action(at: CGPoint(x: 999, y: 300), last: nil, isOpen: false), .armOpen)
        XCTAssertEqual(t.action(at: CGPoint(x: 990, y: 300), last: nil, isOpen: false), .cancelOpen)
    }

    func testClosedFlickOutThroughRightEdgeArms() {
        XCTAssertEqual(t.action(at: nil, last: CGPoint(x: 985, y: 300), isOpen: false), .armOpen)
        XCTAssertEqual(t.action(at: nil, last: CGPoint(x: 500, y: 300), isOpen: false), .cancelOpen)
        XCTAssertEqual(t.action(at: nil, last: CGPoint(x: 985, y: -5), isOpen: false), .cancelOpen, "left via toolbar")
    }

    func testOpenStaysWhileInsidePanelOrNearEdge() {
        XCTAssertEqual(t.action(at: CGPoint(x: 700, y: 300), last: nil, isOpen: true), .none)
        XCTAssertEqual(t.action(at: CGPoint(x: 985, y: 700), last: nil, isOpen: true), .none, "near edge below area")
        XCTAssertEqual(t.action(at: nil, last: CGPoint(x: 700, y: 300), isOpen: true), .none, "left the area: no hover info")
    }

    func testOpenClosesWhenCursorMovesAway() {
        XCTAssertEqual(t.action(at: CGPoint(x: 599, y: 300), last: nil, isOpen: true), .close)
        XCTAssertEqual(t.action(at: CGPoint(x: 700, y: 700), last: nil, isOpen: true), .close, "below panel, not at edge")
    }
}
