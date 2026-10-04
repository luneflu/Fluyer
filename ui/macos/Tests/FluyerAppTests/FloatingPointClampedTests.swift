import XCTest
@testable import FluyerApp

final class FloatingPointClampedTests: XCTestCase {
    func testConfinesToRange() {
        XCTAssertEqual(Float(0.5).clamped(to: 0...1), 0.5)
        XCTAssertEqual(Float(-3).clamped(to: 0...1), 0)
        XCTAssertEqual(Float(4).clamped(to: 0...1), 1)
        XCTAssertEqual(Float(0).clamped(to: 0...1), 0)
        XCTAssertEqual(Float(1).clamped(to: 0...1), 1)
    }

    func testCollapsesNaNInsteadOfPropagating() {
        // `Float.nan` survives `min`/`max`, and `UInt64(Float.nan)` traps.
        XCTAssertEqual(Float.nan.clamped(to: 0...1), 0)
    }

    func testInfinitiesPinToTheRangeEnds() {
        XCTAssertEqual(Float.infinity.clamped(to: 0...1), 1)
        XCTAssertEqual((-Float.infinity).clamped(to: 0...1), 0)
    }

    func testClampedValuesAreAlwaysConvertibleToInteger() {
        for raw in [Float.nan, .infinity, -.infinity, -1, 0, 0.5, 1, 2] {
            XCTAssertNoThrow(UInt64(raw.clamped(to: 0...1)))
        }
    }
}
