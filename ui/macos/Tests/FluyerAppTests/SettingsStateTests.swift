import XCTest
@testable import FluyerApp

/// The file, the JSON shape, folder normalization and the volume clamp all live in
/// the core (`crates/fluyer_core/src/services/settings.rs` and its tests). These
/// cover what is left in Swift: the defaults a view sees before the core answers,
/// remembering folders while no engine is attached, and the enum mapping.
@MainActor
final class SettingsStateTests: XCTestCase {
    func testDefaultsMatchTheCoreDefaults() {
        let s = SettingsState()
        XCTAssertTrue(s.musicFolders.isEmpty)
        XCTAssertTrue(s.animatedBackground)
        XCTAssertEqual(s.backdropSource, .artwork)
        XCTAssertTrue(s.discordRpc)
        XCTAssertEqual(s.volume, 1.0)
    }

    func testAddFoldersRemembersPathsWithoutAnEngineAndReturnsTheNewOnes() {
        let s = SettingsState()
        XCTAssertEqual(s.addFolders(["/Music/", "/Other"]), ["/Music/", "/Other"])
        XCTAssertEqual(s.musicFolders, ["/Music/", "/Other"])
    }

    func testRemoveFolderMatchesTrailingSlashAndReturnsTheStoredSpelling() {
        let s = SettingsState()
        s.addFolders(["/Music"])
        XCTAssertEqual(s.removeFolder("/Music/"), "/Music")
        XCTAssertNil(s.removeFolder("/Music"))
        XCTAssertTrue(s.musicFolders.isEmpty)
    }

    func testRemoveFolderKeepsTheRoot() {
        let s = SettingsState()
        s.addFolders(["/"])
        XCTAssertEqual(s.removeFolder("/"), "/")
    }

    func testBackdropSourceRoundTripsThroughItsRawValue() {
        for source in BackdropSource.allCases {
            XCTAssertEqual(BackdropSource(rawValue: source.rawValue), source)
        }
        XCTAssertNil(BackdropSource(rawValue: "shader"))
    }
}
