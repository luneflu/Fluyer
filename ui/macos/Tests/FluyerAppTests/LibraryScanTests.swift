import XCTest
import FluyerCore
@testable import FluyerApp

/// End-to-end cover for the library scan, through the real core.
///
/// This needs audio metadata extraction, so it is skipped when no sample file is
/// available rather than failing. `SAMPLE_AUDIO_PATH` is the only knob.
final class LibraryScanTests: XCTestCase {
    private static let sampleAudioPath = "/Users/alvindimas05/UTM Shared Folder/1 - The Calling.flac"

    private struct Sandbox {
        let root: URL
        let music: URL
        let engine: FluyerAppEngine

        init() throws {
            let fileManager = FileManager.default
            root = fileManager.temporaryDirectory
                .appendingPathComponent("fluyer-scan-\(UUID().uuidString)")
            music = root.appendingPathComponent("Music")

            let dataDir = root.appendingPathComponent("data")
            let cacheDir = root.appendingPathComponent("cache")
            for directory in [music, dataDir, cacheDir] {
                try fileManager.createDirectory(at: directory, withIntermediateDirectories: true)
            }

            guard fileManager.fileExists(atPath: LibraryScanTests.sampleAudioPath) else {
                throw XCTSkip("no sample audio at \(LibraryScanTests.sampleAudioPath)")
            }
            try fileManager.copyItem(
                atPath: LibraryScanTests.sampleAudioPath,
                toPath: music.appendingPathComponent("sample.flac").path
            )

            engine = try FluyerAppEngine(
                dataDir: dataDir.path,
                cacheDir: cacheDir.path,
                listener: nil
            )
        }

        func cleanUp() {
            try? FileManager.default.removeItem(at: root)
        }
    }

    /// Regression: `repo::upsert_music_batch` used `ON CONFLICT(path)` against a schema
    /// with no unique index on `path`, so every insert failed and the error was
    /// discarded. The scan reported success and left the library empty.
    func testScanPopulatesTheLibrary() async throws {
        let sandbox = try Sandbox()
        defer { sandbox.cleanUp() }

        XCTAssertEqual(sandbox.engine.getTrackCount(), 0)

        sandbox.engine.scanDirectories(directories: [sandbox.music.path])

        try await waitForTracks(sandbox.engine, expected: 1)
        XCTAssertEqual(sandbox.engine.getAlbumCount(), 1)

        let track = try XCTUnwrap(sandbox.engine.getTrackView(index: 0))
        XCTAssertTrue(track.path.hasSuffix("sample.flac"))
        XCTAssertGreaterThan(track.durationMs, 0, "metadata extraction should have run")
    }

    /// A second scan of the same folder must update in place, not duplicate rows.
    func testRescanningIsIdempotent() async throws {
        let sandbox = try Sandbox()
        defer { sandbox.cleanUp() }

        sandbox.engine.scanDirectories(directories: [sandbox.music.path])
        try await waitForTracks(sandbox.engine, expected: 1)

        sandbox.engine.scanDirectories(directories: [sandbox.music.path])
        // Unchanged mtime means the file is skipped entirely; give the second pass a
        // moment so a late-inserted duplicate would show up here.
        try await Task.sleep(nanoseconds: 1_000_000_000)

        XCTAssertEqual(sandbox.engine.getTrackCount(), 1)
    }

    /// A folder with no supported audio is not an error, and must not wipe the library.
    func testScanningAnEmptyFolderLeavesTheLibraryIntact() async throws {
        let sandbox = try Sandbox()
        defer { sandbox.cleanUp() }

        sandbox.engine.scanDirectories(directories: [sandbox.music.path])
        try await waitForTracks(sandbox.engine, expected: 1)

        let empty = sandbox.root.appendingPathComponent("Empty")
        try FileManager.default.createDirectory(at: empty, withIntermediateDirectories: true)
        sandbox.engine.scanDirectories(directories: [empty.path])
        try await Task.sleep(nanoseconds: 500_000_000)

        XCTAssertEqual(sandbox.engine.getTrackCount(), 1)
    }

    /// Removing a library folder drops its tracks synchronously; the queue view
    /// mirrors what was enqueued and clears on demand.
    func testQueueAndRemoveFolderThroughTheCore() async throws {
        let sandbox = try Sandbox()
        defer { sandbox.cleanUp() }

        sandbox.engine.scanDirectories(directories: [sandbox.music.path])
        try await waitForTracks(sandbox.engine, expected: 1)

        sandbox.engine.queueAlbum(index: 0)
        let queue = sandbox.engine.getQueueView()
        XCTAssertEqual(queue.count, 1)
        XCTAssertEqual(queue.first?.index, 0, "row index is the queue position")

        sandbox.engine.queueClear()
        XCTAssertTrue(sandbox.engine.getQueueView().isEmpty)

        sandbox.engine.removeFolder(directory: sandbox.music.path)
        XCTAssertEqual(sandbox.engine.getTrackCount(), 0)
    }

    /// The scan runs on a background runtime, so results are not available on return.
    private func waitForTracks(
        _ engine: FluyerAppEngine,
        expected: UInt64,
        timeout: TimeInterval = 20
    ) async throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if engine.getTrackCount() == expected { return }
            try await Task.sleep(nanoseconds: 50_000_000)
        }
        XCTFail("timed out waiting for \(expected) track(s), got \(engine.getTrackCount())")
        throw XCTSkip("scan timed out")
    }
}
