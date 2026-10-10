import XCTest
import FluyerCore
@testable import FluyerApp

@MainActor
final class LibraryFilterStateTests: XCTestCase {
    private func makeSelection(libraryTrackCount: Int = 0) -> (LibraryState, LibraryFilterState) {
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
        return (library, LibraryFilterState(library: library))
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

    func testTrackSortOrdersNaturallyAndReverses() {
        let (library, selection) = makeSelection(libraryTrackCount: 3)
        library.tracks[0].title = "Track 10"
        library.tracks[1].title = "track 2"
        library.tracks[2].title = "Alpha"
        library.tracks[0].durationMs = 300
        library.tracks[2].durationMs = 100

        XCTAssertEqual(selection.displayedTracks.map(\.index), [0, 1, 2]) // library order
        selection.trackSort = .title
        XCTAssertEqual(selection.displayedTracks.map(\.index), [2, 1, 0]) // "2" before "10"
        selection.sortAscending = false
        XCTAssertEqual(selection.displayedTracks.map(\.index), [0, 1, 2])
        selection.trackSort = .duration
        XCTAssertEqual(selection.displayedTracks.map(\.index), [1, 0, 2]) // 1000, 300, 100
    }

    func testAlbumSortByYearDescending() {
        let (library, selection) = makeSelection()
        library.albums = [("A", "1999"), ("B", "2010"), ("C", "")].enumerated().map { i, album in
            AlbumCardViewModel(index: UInt64(i), name: album.0, artist: "", year: album.1, trackCount: 1, trackCountLabel: "")
        }
        selection.albumSort = .year
        selection.sortAscending = false

        XCTAssertEqual(selection.displayedAlbums.map(\.name), ["B", "A", "C"])
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

    func testAlbumQueryFiltersNameAndArtist() {
        let (library, selection) = makeSelection()
        library.albums = [
            AlbumCardViewModel(index: 0, name: "Discovery", artist: "Daft Punk", year: "", trackCount: 1, trackCountLabel: ""),
            AlbumCardViewModel(index: 1, name: "Melody AM", artist: "Röyksopp", year: "", trackCount: 1, trackCountLabel: "")
        ]

        XCTAssertEqual(selection.displayedAlbums.count, 2)
        selection.query = "royk"
        XCTAssertEqual(selection.displayedAlbums.map(\.index), [1])
        selection.query = "DISCO"
        XCTAssertEqual(selection.displayedAlbums.map(\.index), [0])
    }

    /// Picking an album in the album grid returns to its tracks.
    func testSelectFromAlbumGridSwitchesToTracks() {
        let (_, selection) = makeSelection(libraryTrackCount: 1)
        selection.mode = .albums

        selection.select(0)

        XCTAssertEqual(selection.mode, .tracks)
    }

    func testQueryFiltersTitleArtistAlbumInsensitively() {
        let (library, selection) = makeSelection(libraryTrackCount: 3)
        library.tracks[1].title = "Café del Mar"
        library.tracks[2].artist = "Röyksopp"

        selection.query = "  cafe "
        XCTAssertEqual(selection.displayedTracks.map(\.index), [1])

        selection.query = "ROYK"
        XCTAssertEqual(selection.displayedTracks.map(\.index), [2])

        selection.query = "album"
        XCTAssertEqual(selection.displayedTracks.count, 3)

        selection.query = "zzz"
        XCTAssertTrue(selection.displayedTracks.isEmpty)

        selection.query = ""
        XCTAssertEqual(selection.displayedTracks.count, 3)
    }
}
