import XCTest
import FluyerCore
@testable import FluyerApp

@MainActor
final class AlbumSelectionTests: XCTestCase {
    private func makeSelection(libraryTrackCount: Int = 0) -> (LibraryState, AlbumSelection) {
        let library = LibraryState()
        library.tracks = (0..<libraryTrackCount).map { index in
            TrackItemViewModel(
                index: UInt64(index),
                path: "/music/\(index).flac",
                title: "Track \(index)",
                artist: "Artist",
                album: "Album",
                durationMs: 1_000,
                durationFormatted: "0:01",
                isCurrent: false
            )
        }
        return (library, AlbumSelection(library: library))
    }

    func testStartsInactiveShowingTheWholeLibrary() {
        let (_, selection) = makeSelection(libraryTrackCount: 3)

        XCTAssertFalse(selection.isActive)
        XCTAssertNil(selection.index)
        XCTAssertEqual(selection.displayedTracks.count, 3)
    }

    func testClearReturnsToTheLibrary() {
        let (_, selection) = makeSelection(libraryTrackCount: 3)
        selection.select(1)

        selection.clear()

        XCTAssertFalse(selection.isActive)
        XCTAssertNil(selection.index)
        XCTAssertNil(selection.detail)
        XCTAssertEqual(selection.displayedTracks.count, 3)
    }

    /// Regression: `index` and `detail` used to be independent optionals. When the
    /// detail fetch failed the old `displayedTracks` still showed the library while
    /// the header claimed an album was open.
    func testFailedDetailFetchStillFallsBackToTheLibrary() {
        let (_, selection) = makeSelection(libraryTrackCount: 2)
        // No engine attached, so `getAlbumDetail` cannot succeed.
        selection.select(0)

        XCTAssertTrue(selection.isActive)
        XCTAssertNil(selection.detail)
        XCTAssertEqual(selection.displayedTracks.count, 2)
    }

    func testNegativeRowIsRejected() {
        let (_, selection) = makeSelection(libraryTrackCount: 1)
        // Must not trap on `UInt64(-1)`.
        selection.playTrack(atRow: -1)
    }

    func testAlbumCommandsAreInertWithoutASelection() {
        let (_, selection) = makeSelection()

        selection.playSelected()
        selection.queueSelected()
        selection.shuffleSelected()
    }

    func testReloadClearsADetailForAClearedSelection() {
        let (_, selection) = makeSelection(libraryTrackCount: 1)
        selection.select(0)

        selection.reload()

        XCTAssertEqual(selection.index, 0)
        XCTAssertNil(selection.detail)
    }
}
