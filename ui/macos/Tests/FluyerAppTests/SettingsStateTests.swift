import XCTest
@testable import FluyerApp

@MainActor
final class SettingsStateTests: XCTestCase {
    private var dir: URL!
    private var url: URL { dir.appendingPathComponent(SettingsState.fileName) }

    override func setUp() async throws {
        dir = FileManager.default.temporaryDirectory.appendingPathComponent("fluyer-settings-\(UUID().uuidString)")
    }

    override func tearDown() async throws {
        try? FileManager.default.removeItem(at: dir)
    }

    func testRoundTripsEveryField() {
        let s = SettingsState(url: url)
        s.addFolders(["/Music/", "/Other"])
        s.animatedBackground = false
        s.discordRpc = false
        s.volume = 0.4

        let reloaded = SettingsState(url: url)
        XCTAssertEqual(reloaded.musicFolders, ["/Music", "/Other"])
        XCTAssertFalse(reloaded.animatedBackground)
        XCTAssertFalse(reloaded.discordRpc)
        XCTAssertEqual(reloaded.volume, 0.4, accuracy: 0.0001)
    }

    func testAddFoldersDedupesIgnoringTrailingSlash() {
        let s = SettingsState()
        XCTAssertEqual(s.addFolders(["/Music", "/Music/", " "]), ["/Music"])
        XCTAssertEqual(s.addFolders(["/Music//"]), [])
        XCTAssertEqual(s.musicFolders, ["/Music"])
    }

    func testRootStaysRoot() {
        XCTAssertEqual(SettingsState.normalize("/"), "/")
    }

    func testRemoveReturnsStoredSpellingOrNil() {
        let s = SettingsState()
        s.addFolders(["/Music"])
        XCTAssertEqual(s.removeFolder("/Music/"), "/Music")
        XCTAssertNil(s.removeFolder("/Music"))
        XCTAssertTrue(s.musicFolders.isEmpty)
    }

    func testCorruptFileIsKeptAsBadAndDefaultsUsed() throws {
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        try Data("{not json".utf8).write(to: url)

        let s = SettingsState(url: url)
        XCTAssertTrue(s.animatedBackground)
        XCTAssertTrue(s.musicFolders.isEmpty)
        let bad = url.appendingPathExtension("bad")
        XCTAssertEqual(try String(contentsOf: bad, encoding: .utf8), "{not json")
    }

    func testPartialFileKeepsDefaultsAndClampsVolume() throws {
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        try Data(#"{"volume": 7}"#.utf8).write(to: url)

        let s = SettingsState(url: url)
        XCTAssertEqual(s.volume, 1.0)
        XCTAssertTrue(s.discordRpc)
    }
}
