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

    var body: some View {
        if state.library.albums.isEmpty {
            EmptyView()
        } else {
            GeometryReader { geo in
                let metrics = Metrics(width: geo.size.width, dpr: dpr)
                ScrollView(.horizontal, showsIndicators: false) {
                    LazyHStack(spacing: kCardGap) {
                        ForEach(state.library.albums, id: \.index) { album in
                            AlbumCarouselCard(state: state, album: album, metrics: metrics)
                                .hiddenBySidebar()
                        }
                    }
                    .scrollTargetLayout()
                }
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

        init(width: CGFloat, dpr: Double) {
            let itemWidth = Self.itemWidth(width: width, dpr: dpr)
            let coverSize = max(16, itemWidth - kCardGap)
            self.itemWidth = itemWidth
            self.coverSize = coverSize
            self.height = coverSize + kCardLabelHeight
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

// MARK: - Card

private struct AlbumCarouselCard: View {
    @Bindable var state: AppState
    let album: AlbumCardViewModel
    let metrics: AlbumCarouselView.Metrics

    @State private var thumbnail: NSImage?

    /// Requested at 2x for Retina, which is also what makes this key distinct from
    /// the 88px album thumbnail the track rows ask for.
    private var pixelSize: Int { max(16, Int(metrics.coverSize * 2)) }

    private var cacheKey: String { ThumbnailKey.album(album.index, px: pixelSize) }

    private var isSelected: Bool { state.selection.index == Int(album.index) }

    var body: some View {
        VStack(alignment: .leading, spacing: 5) {
            cover
            labels
        }
        .frame(width: metrics.coverSize)
        .contentShape(Rectangle())
        .onTapGesture {
            state.selection.select(Int(album.index))
        }
        .task(id: cacheKey) {
            guard let engine = state.engine else { return }
            let maxSize = UInt32(pixelSize)
            let data = await engine.loadAlbumThumbnail(index: album.index, maxSize: maxSize)
            guard let data, !data.isEmpty else { return }
            thumbnail = await ThumbnailStore.shared.image(key: cacheKey) { data }
        }
    }

    @ViewBuilder
    private var cover: some View {
        ZStack {
            if let thumbnail {
                Image(nsImage: thumbnail)
                    .resizable()
                    .aspectRatio(1, contentMode: .fill)
            } else {
                Rectangle()
                    .fill(Color.white.opacity(0.08))
                    .overlay(
                        Image(systemName: "music.note")
                            .font(.system(size: metrics.coverSize * 0.25))
                            .foregroundColor(.white.opacity(0.3))
                    )
            }
        }
        .frame(width: metrics.coverSize, height: metrics.coverSize)
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
        .frame(width: metrics.coverSize, alignment: .leading)
    }
}
