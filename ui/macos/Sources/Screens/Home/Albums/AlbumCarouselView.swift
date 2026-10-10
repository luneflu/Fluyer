import SwiftUI
import FluyerCore

/// Horizontal strip of album covers; tapping one drills the grid into that album.
struct AlbumCarouselView: View {
    let filter: LibraryFilterState
    let covers: CoverState
    @State private var carouselHeight: CGFloat = 210
    @State private var isHovered = false

    var body: some View {
        let albums = filter.displayedAlbums
        if albums.isEmpty {
            EmptyView()
        } else {
            GeometryReader { geo in
                let metrics = AlbumMetrics(width: geo.size.width, dpr: AlbumMetrics.screenScale)
                ScrollViewReader { proxy in
                    ScrollView(.horizontal, showsIndicators: false) {
                        LazyHStack(spacing: AlbumMetrics.cardGap) {
                            ForEach(albums, id: \.index) { album in
                                AlbumCard(filter: filter, covers: covers, album: album, coverSize: metrics.coverSize,
                                          isListHovered: isHovered)
                                    .id(album.index)
                                    .hiddenBySidebar()
                            }
                        }
                        .scrollTargetLayout()
                    }
                    // Picked in the album grid: land on it (legacy `albumsUi.scrollIndex`).
                    .onAppear {
                        if let index = filter.index { proxy.scrollTo(UInt64(index), anchor: .leading) }
                    }
                }
                .onHover { isHovered = $0 }
                // Edge padding sits outside the scroll view so its bounds clip the
                // next card: n covers + (n−1) gaps fill the visible width exactly,
                // and `contentMargins` would let the (n+1)th peek into the margin.
                .scrollTargetBehavior(.viewAligned)
                .padding(.horizontal, AlbumMetrics.edge)
                .onAppear { carouselHeight = metrics.height }
                .onChange(of: metrics.height) { _, newHeight in carouselHeight = newHeight }
            }
            .frame(height: carouselHeight)
        }
    }
}
