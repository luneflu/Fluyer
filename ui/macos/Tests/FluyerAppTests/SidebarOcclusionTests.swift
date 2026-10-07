import XCTest
@testable import FluyerApp

final class SidebarOcclusionTests: XCTestCase {
    /// Mirrors `itemRight > viewportWidth - sidebarWidth + extraToleranceWidth`.
    func testHidesOnlyPastTolerance() {
        XCTAssertFalse(SidebarOcclusion.isCovered(itemMaxX: 1000, sidebarMinX: 990))
        XCTAssertTrue(SidebarOcclusion.isCovered(itemMaxX: 1000.5, sidebarMinX: 990))
        XCTAssertFalse(SidebarOcclusion.isCovered(itemMaxX: 5000, sidebarMinX: .infinity))
    }
}
