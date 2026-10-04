import XCTest
@testable import FluyerApp

final class ThumbnailKeyTests: XCTestCase {
    /// The album carousel requests ~400pt covers while an album's track rows request
    /// 88pt ones. They must not share a cache entry, or the carousel renders the row
    /// bitmap scaled up.
    func testAlbumKeyIncludesPixelSize() {
        let carousel = ThumbnailKey.album(3, px: 800)
        let row = ThumbnailKey.album(3, px: 88)
        XCTAssertNotEqual(carousel, row)
    }

    func testDistinctIndicesAndSizesNeverCollide() {
        var keys: Set<String> = []
        for index in UInt64(0)..<4 {
            for pixels in [88, 800] {
                keys.insert(ThumbnailKey.track(index, px: pixels))
                keys.insert(ThumbnailKey.album(index, px: pixels))
            }
        }
        XCTAssertEqual(keys.count, 4 * 2 * 2)
    }

    func testTrackAndAlbumNamespacesDoNotOverlap() {
        XCTAssertFalse(ThumbnailKey.track(7, px: 88).hasPrefix(ThumbnailKey.albumPrefix(7)))
        XCTAssertFalse(ThumbnailKey.album(7, px: 88).hasPrefix(ThumbnailKey.trackPrefix(7)))
    }

    func testPrefixesCoverEveryCachedSize() {
        for pixels in [88, 176, 800] {
            XCTAssertTrue(ThumbnailKey.album(2, px: pixels).hasPrefix(ThumbnailKey.albumPrefix(2)))
            XCTAssertTrue(ThumbnailKey.track(2, px: pixels).hasPrefix(ThumbnailKey.trackPrefix(2)))
        }
        XCTAssertFalse(ThumbnailKey.album(2, px: 88).hasPrefix(ThumbnailKey.albumPrefix(3)))
    }

    /// The current cover is keyed on the track path so a library rescan that renumbers
    /// tracks cannot serve the previous song's artwork.
    func testCurrentKeyTracksIdentityNotIndex() {
        let first = ThumbnailKey.current(side: 80, path: "/music/a.flac")
        let second = ThumbnailKey.current(side: 80, path: "/music/b.flac")
        XCTAssertNotEqual(first, second)
        XCTAssertEqual(first, ThumbnailKey.current(side: 80, path: "/music/a.flac"))
        XCTAssertNotEqual(first, ThumbnailKey.current(side: 720, path: "/music/a.flac"))
    }
}
