import XCTest
import FluyerCore
@testable import FluyerApp

final class PlaybackIconsTests: XCTestCase {
    func testRepeatIconCoversEveryMode() {
        XCTAssertEqual(PlaybackIcons.repeatIcon(.one), "repeat.1")
        XCTAssertEqual(PlaybackIcons.repeatIcon(.all), "repeat")
        XCTAssertEqual(PlaybackIcons.repeatIcon(.none), "repeat")
    }

    func testVolumeIconBands() {
        XCTAssertEqual(PlaybackIcons.volume(0), "speaker.slash.fill")
        XCTAssertEqual(PlaybackIcons.volume(0.001), "speaker.slash.fill")
        XCTAssertEqual(PlaybackIcons.volume(0.002), "speaker.wave.1.fill")
        XCTAssertEqual(PlaybackIcons.volume(0.329), "speaker.wave.1.fill")
        XCTAssertEqual(PlaybackIcons.volume(0.33), "speaker.wave.2.fill")
        XCTAssertEqual(PlaybackIcons.volume(0.659), "speaker.wave.2.fill")
        XCTAssertEqual(PlaybackIcons.volume(0.66), "speaker.wave.3.fill")
        XCTAssertEqual(PlaybackIcons.volume(1), "speaker.wave.3.fill")
    }
}
