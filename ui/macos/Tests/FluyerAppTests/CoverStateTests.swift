import XCTest
@testable import FluyerApp

@MainActor
final class CoverStateTests: XCTestCase {
    /// Before the core starts (or if it failed), views must get placeholders, not crashes.
    func testNoEngineReturnsNil() async {
        let covers = CoverState()
        let track = await covers.track(0, px: 88)
        let album = await covers.album(0, px: 88)
        let current = await covers.current(px: 88, path: "/x.flac")
        let backdrop = await covers.backdrop(source: .artwork, hasTrack: true)
        XCTAssertNil(track)
        XCTAssertNil(album)
        XCTAssertNil(current)
        XCTAssertNil(backdrop)
        XCTAssertNil(covers.cached(ThumbnailKey.track(0, px: 88)))
    }
}
