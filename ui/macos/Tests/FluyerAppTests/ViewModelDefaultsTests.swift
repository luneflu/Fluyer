import XCTest
import FluyerCore
@testable import FluyerApp

/// The placeholder view models must keep matching the Rust constructors they mirror
/// (`PlayerBarViewModel::default`, `ScanStatusViewModel::idle`), otherwise the window
/// shows a stray title or a blank scan label before the core reports anything.
final class ViewModelDefaultsTests: XCTestCase {
    func testPlayerBarPlaceholder() {
        let placeholder = PlayerBarViewModel.noTrack
        XCTAssertEqual(placeholder.trackIndex, -1)
        XCTAssertEqual(placeholder.title, "No Track")
        XCTAssertEqual(placeholder.artist, "")
        XCTAssertEqual(placeholder.album, "")
        XCTAssertEqual(placeholder.positionMs, 0)
        XCTAssertEqual(placeholder.durationMs, 0)
        XCTAssertEqual(placeholder.progressPct, 0)
        XCTAssertEqual(placeholder.timeLabel, "0:00 / 0:00")
        XCTAssertFalse(placeholder.isPlaying)
        XCTAssertEqual(placeholder.repeatMode, .none)
        XCTAssertFalse(placeholder.isShuffled)
        XCTAssertEqual(placeholder.volume, 1.0)
    }

    func testPlayViewPlaceholder() {
        let placeholder = PlayViewModel.empty
        XCTAssertNil(placeholder.track)
        XCTAssertTrue(placeholder.lyrics.isEmpty)
        XCTAssertEqual(placeholder.currentLyricIndex, -1)
        XCTAssertEqual(placeholder.palette, [ColorRgb(r: 28, g: 28, b: 36)])
    }

    func testScanStatusPlaceholder() {
        let placeholder = ScanStatusViewModel.idle
        XCTAssertFalse(placeholder.isScanning)
        XCTAssertEqual(placeholder.current, 0)
        XCTAssertEqual(placeholder.total, 0)
        XCTAssertEqual(placeholder.progressPct, 0)
        XCTAssertEqual(placeholder.statusLabel, "Ready")
    }
}
