import XCTest
@testable import FluyerApp

final class RemoteCoverSizingTests: XCTestCase {
    /// Cards ask the CDN for their own pixel size; only the play view keeps the
    /// full-size URL the core hands out.
    func testRewritesEverySizeTokenShape() {
        XCTAssertEqual(
            RemoteCover.sized("https://yt3.googleusercontent.com/abc=w1600-h1600-l90-rj", px: 680),
            "https://yt3.googleusercontent.com/abc=w680-h680-l90-rj"
        )
        XCTAssertEqual(
            RemoteCover.sized("https://yt3.ggpht.com/abc=s576", px: 88),
            "https://yt3.ggpht.com/abc=w88-h88"
        )
        XCTAssertEqual(
            RemoteCover.sized("https://yt3.ggpht.com/abc=s192-c-k-c0x00ffffff-no-rj", px: 88),
            "https://yt3.ggpht.com/abc=w88-h88-c-k-c0x00ffffff-no-rj"
        )
        XCTAssertEqual(
            RemoteCover.sized("https://example.com/cover.jpg", px: 88),
            "https://example.com/cover.jpg"
        )
    }
}
