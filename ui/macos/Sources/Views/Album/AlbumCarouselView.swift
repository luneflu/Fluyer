import SwiftUI
import AppKit
import FluyerCore

/// Number of album cards that fit across the window, as a fraction of its width.
///
/// ponytail: transcribed from the deleted `AlbumListCtrl::RESPONSIVE_RULES`. The
/// HiDPI rows are listed first so a Retina display matches before the 1x row with the
/// same minimum width.
private struct ResponsiveRule {
    let minWidth: CGFloat
    let minDpr: Double
    let widthRatio: CGFloat
}

private let kResponsiveRules: [ResponsiveRule] = [
    ResponsiveRule(minWidth: 1536, minDpr: 1.01, widthRatio: 0.142857), // hdpi 2xl → 7 items
    ResponsiveRule(minWidth: 1280, minDpr: 1.01, widthRatio: 0.16667),  // xl-hdpi → 6 items
    ResponsiveRule(minWidth: 1024, minDpr: 1.01, widthRatio: 0.2),      // lg-hdpi → 5 items
    ResponsiveRule(minWidth: 768,  minDpr: 1.01, widthRatio: 0.25),     // md-hdpi → 4 items
    ResponsiveRule(minWidth: 640,  minDpr: 1.01, widthRatio: 0.33334),  // sm-hdpi → 3 items

    ResponsiveRule(minWidth: 1536, minDpr: 0.0,  widthRatio: 0.125),    // 2xl → 8 items
    ResponsiveRule(minWidth: 1440, minDpr: 0.0,  widthRatio: 0.142857), // 1440 → 7 items
    ResponsiveRule(minWidth: 1280, minDpr: 0.0,  widthRatio: 0.16667),  // xl → 6 items
    ResponsiveRule(minWidth: 1024, minDpr: 0.0,  widthRatio: 0.2),      // lg → 5 items
    ResponsiveRule(minWidth: 768,  minDpr: 0.0,  widthRatio: 0.25),     // md → 4 items
    ResponsiveRule(minWidth: 640,  minDpr: 0.0,  widthRatio: 0.33334)   // sm → 3 items
]

private let kCardGap: CGFloat = 12
/// Same 16pt side gutter as `MusicGridView`.
private let kCarouselEdge: CGFloat = 16
private let kCardLabelHeight: CGFloat = 52

/// Horizontal strip of album covers; tapping one drills the grid into that album.
struct AlbumCarouselView: View {
    @Bindable var state: AppState
    @State private var carouselHeight: CGFloat = 210
    @State private var isHovered = false

    var body: some View {
        let albums = state.selection.displayedAlbums
        if albums.isEmpty {
            EmptyView()
        } else {
            GeometryReader { geo in
                let metrics = Metrics(width: geo.size.width, dpr: dpr)
                ScrollViewReader { proxy in
                    ScrollView(.horizontal, showsIndicators: false) {
                        LazyHStack(spacing: kCardGap) {
                            ForEach(albums, id: \.index) { album in
                                AlbumCard(state: state, album: album, coverSize: metrics.coverSize,
                                          isListHovered: isHovered)
                                    .id(album.index)
                                    .hiddenBySidebar()
                            }
                        }
                        .scrollTargetLayout()
                    }
                    // Picked in the album grid: land on it (legacy `albumsUi.scrollIndex`).
                    .onAppear {
                        if let index = state.selection.index { proxy.scrollTo(UInt64(index), anchor: .leading) }
                    }
                }
                .onHover { isHovered = $0 }
                // Edge padding sits outside the scroll view so its bounds clip the
                // next card: n covers + (n−1) gaps fill the visible width exactly,
                // and `contentMargins` would let the (n+1)th peek into the margin.
                .scrollTargetBehavior(.viewAligned)
                .padding(.horizontal, kCarouselEdge)
                .onAppear { carouselHeight = metrics.height }
                .onChange(of: metrics.height) { _, newHeight in carouselHeight = newHeight }
            }
            .frame(height: carouselHeight)
        }
    }

    private var dpr: Double { Self.screenScale }

    private static var screenScale: Double {
        Double(NSScreen.main?.backingScaleFactor ?? 2.0)
    }

    /// Sidebar width for a window `width`: two album slots minus the 12pt outer
    /// padding on each side (`Sidebar.svelte`: `sidebarStore.width - 24`, where
    /// `sidebarStore.width = itemWidth * 2` in `useAlbumList`).
    static func sidebarWidth(forWidth width: CGFloat) -> CGFloat {
        max(1, Metrics.itemWidth(width: width, dpr: screenScale) * 2 - 24)
    }

    /// Resolved card geometry for the current window size.
    ///
    /// The rule's ratio splits `width - 2·edge + gap` into whole slots, so n covers,
    /// n−1 gaps and both edges fill the width exactly (Windows `ItemWidth` does the
    /// same). Scrolling snaps to cards, and the scroll range is a whole number of
    /// slots, so the last position lines up as cleanly as the first.
    struct Metrics: Equatable {
        /// Cover plus one gap.
        let itemWidth: CGFloat
        let coverSize: CGFloat
        let height: CGFloat
        /// Cards per row in the album grid (`Math.round(1 / widthRatio)`).
        let columnCount: Int

        init(width: CGFloat, dpr: Double) {
            let itemWidth = Self.itemWidth(width: width, dpr: dpr)
            let coverSize = max(16, itemWidth - kCardGap)
            self.itemWidth = itemWidth
            self.coverSize = coverSize
            self.height = coverSize + kCardLabelHeight
            self.columnCount = max(1, Int((max(1, width - kCarouselEdge * 2 + kCardGap) / itemWidth).rounded()))
        }

        static func itemWidth(width: CGFloat, dpr: Double) -> CGFloat {
            let available = max(1, width - kCarouselEdge * 2 + kCardGap)
            for rule in kResponsiveRules where width >= rule.minWidth && dpr >= rule.minDpr {
                return rule.widthRatio * available
            }
            return 0.5 * available
        }
    }
}

// MARK: - Grid

/// Full-height album grid (`AlbumList.svelte` in album mode). Same card width as
/// the carousel, so switching modes keeps every cover the same size.
struct AlbumGridView: View {
    @Bindable var state: AppState
    @State private var isHovered = false

    var body: some View {
        let albums = state.selection.displayedAlbums
        if albums.isEmpty {
            Text(state.library.albums.isEmpty ? "No albums in library" : "No albums found")
                .font(.system(size: 14))
                .foregroundColor(.white.opacity(0.5))
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else {
            GeometryReader { geo in
                let metrics = AlbumCarouselView.Metrics(width: geo.size.width, dpr: Double(NSScreen.main?.backingScaleFactor ?? 2))
                let columns = Array(repeating: GridItem(.fixed(metrics.coverSize), spacing: kCardGap, alignment: .top),
                                    count: metrics.columnCount)
                ScrollView {
                    LazyVGrid(columns: columns, spacing: kCardGap) {
                        ForEach(albums, id: \.index) { album in
                            AlbumCard(state: state, album: album, coverSize: metrics.coverSize,
                                      isListHovered: isHovered)
                                .hiddenBySidebar()
                        }
                    }
                    .padding(.horizontal, kCarouselEdge)
                    .padding(.vertical, 8)
                }
                .scrollIndicators(.hidden)
                .onHover { isHovered = $0 }
            }
        }
    }
}

// MARK: - Card

/// One album cover with its labels (`AlbumItem.svelte`). Click opens the album,
/// right-click plays / queues / shuffles it. While another album is open, the rest
/// dim until the pointer enters the list.
struct AlbumCard: View {
    @Bindable var state: AppState
    let album: AlbumCardViewModel
    let coverSize: CGFloat
    var isListHovered = false

    @State private var thumbnail: NSImage?
    @State private var isHovered = false

    /// Requested at 2x for Retina, which is also what makes this key distinct from
    /// the 88px album thumbnail the track rows ask for.
    private var pixelSize: Int { max(16, Int(coverSize * 2)) }

    private var cacheKey: String { ThumbnailKey.album(album.index, px: pixelSize) }

    private var isSelected: Bool { state.selection.index == Int(album.index) }

    private var isDimmed: Bool { state.selection.isActive && !isSelected && !isListHovered }

    var body: some View {
        VStack(alignment: .leading, spacing: 5) {
            cover
            labels
        }
        .frame(width: coverSize)
        .opacity(isDimmed ? 0.4 : 1)
        .animation(.easeInOut(duration: 0.3), value: isDimmed)
        .contentShape(Rectangle())
        .onHover { isHovered = $0 }
        .onTapGesture {
            state.selection.select(Int(album.index))
        }
        .contextMenu {
            Button("Play") { state.selection.playAlbum(Int(album.index)) }
            Button("Add to Queue") { state.selection.queueAlbum(Int(album.index)) }
            Button("Shuffle") { state.selection.shuffleAlbum(Int(album.index)) }
        }
        .accessibilityElement(children: .combine)
        .accessibilityAddTraits(.isButton)
        .accessibilityAddTraits(isSelected ? .isSelected : [])
        .task(id: cacheKey) {
            // Warm cache: show at once. Cold: fade in once decoded (`anim-fade-in`).
            if let hit = ThumbnailStore.shared.peek(cacheKey) {
                thumbnail = hit
                return
            }
            guard let engine = state.engine else { return }
            let maxSize = UInt32(pixelSize)
            let data = await engine.loadAlbumThumbnail(index: album.index, maxSize: maxSize)
            guard let data, !data.isEmpty else { return }
            let image = await ThumbnailStore.shared.image(key: cacheKey) { data }
            withAnimation(.easeIn(duration: 0.3)) { thumbnail = image }
        }
    }

    @ViewBuilder
    private var cover: some View {
        ZStack {
            Rectangle()
                .fill(Color.white.opacity(0.08))
                .overlay(
                    Image(systemName: "music.note")
                        .font(.system(size: coverSize * 0.25))
                        .foregroundColor(.white.opacity(0.3))
                )
            if let thumbnail {
                Image(nsImage: thumbnail)
                    .resizable()
                    .aspectRatio(1, contentMode: .fill)
                    .transition(.opacity)
            }
            // Selected: solid border. Others: border + white wash on hover.
            RoundedRectangle(cornerRadius: 4)
                .fill(Color.white.opacity(isSelected ? 0 : 0.2))
                .strokeBorder(Color.white, lineWidth: 2)
                .opacity(isSelected || isHovered ? 1 : 0)
                .animation(.easeInOut(duration: isHovered ? 0.5 : 0.75), value: isHovered)
        }
        .frame(width: coverSize, height: coverSize)
        .clipShape(RoundedRectangle(cornerRadius: 4))
        .shadow(color: .black.opacity(0.25), radius: 6, x: 0, y: 3)
    }

    private var labels: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(album.name)
                .font(.system(size: 13, weight: isSelected ? .semibold : .medium))
                .foregroundColor(.white)
                .lineLimit(1)

            Text(album.artist)
                .font(.system(size: 11))
                .foregroundColor(.white.opacity(0.6))
                .lineLimit(1)
        }
        .frame(width: coverSize, alignment: .leading)
    }
}
