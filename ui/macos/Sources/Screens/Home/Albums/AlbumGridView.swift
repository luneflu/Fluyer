import SwiftUI
import FluyerCore

/// Full-height album grid (`AlbumList.svelte` in album mode). Same card width as
/// the carousel, so switching modes keeps every cover the same size.
struct AlbumGridView: View {
    let filter: LibraryFilterState
    let library: LibraryState
    let covers: CoverState
    @State private var isHovered = false

    var body: some View {
        let albums = filter.displayedAlbums
        if albums.isEmpty {
            Text(library.albums.isEmpty ? "No albums in library" : "No albums found")
                .font(.system(size: 14))
                .foregroundColor(.white.opacity(0.5))
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else {
            GeometryReader { geo in
                let metrics = AlbumMetrics(width: geo.size.width, dpr: AlbumMetrics.screenScale)
                let columns = Array(repeating: GridItem(.fixed(metrics.coverSize), spacing: AlbumMetrics.cardGap, alignment: .top),
                                    count: metrics.columnCount)
                ScrollView {
                    LazyVGrid(columns: columns, spacing: AlbumMetrics.cardGap) {
                        ForEach(albums, id: \.index) { album in
                            AlbumCard(filter: filter, covers: covers, album: album, coverSize: metrics.coverSize,
                                      isListHovered: isHovered)
                                .hiddenBySidebar()
                        }
                    }
                    .padding(.horizontal, AlbumMetrics.edge)
                    .padding(.vertical, Layout.Albums.gridVerticalPadding)
                }
                .scrollIndicators(.hidden)
                .onHover { isHovered = $0 }
            }
        }
    }
}
