import SwiftUI
import AppKit
import FluyerCore

private struct ResponsiveRule {
    let minWidth: CGFloat
    let minDpr: Double
    let widthRatio: CGFloat
}

// ponytail: matches wxWidgets AlbumListCtrl.cpp RESPONSIVE_RULES exactly
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

public struct AlbumCarouselView: View {
    @Bindable var state: AppState
    @State private var carouselHeight: CGFloat = 210

    public var body: some View {
        if state.albums.isEmpty {
            EmptyView()
        } else {
            GeometryReader { geo in
                let containerWidth = geo.size.width
                let dpr = Double(NSScreen.main?.backingScaleFactor ?? 2.0)
                let itemWidth = computeItemWidth(width: containerWidth, dpr: dpr)
                let padding: CGFloat = 6
                let coverSize = max(16, itemWidth - padding * 2)
                let neededHeight = coverSize + 52

                ScrollView(.horizontal, showsIndicators: false) {
                    LazyHStack(spacing: 0) {
                        ForEach(state.albums, id: \.index) { album in
                            AlbumCarouselCard(
                                state: state,
                                album: album,
                                itemWidth: itemWidth,
                                coverSize: coverSize
                            )
                        }
                    }
                    .padding(.horizontal, 8)
                }
                .onAppear {
                    carouselHeight = neededHeight
                }
                .onChange(of: neededHeight) { _, newH in
                    carouselHeight = newH
                }
            }
            .frame(height: carouselHeight)
        }
    }

    private func computeItemWidth(width: CGFloat, dpr: Double) -> CGFloat {
        for rule in kResponsiveRules {
            if width >= rule.minWidth && dpr >= rule.minDpr {
                return max(1, rule.widthRatio * width)
            }
        }
        return max(1, 0.5 * width)
    }

    @ViewBuilder
    private func albumCard(album: AlbumCardViewModel, itemWidth: CGFloat, coverSize: CGFloat) -> some View {
        AlbumCarouselCard(state: state, album: album, itemWidth: itemWidth, coverSize: coverSize)
    }
}

// MARK: - Card

private struct AlbumCarouselCard: View {
    @Bindable var state: AppState
    let album: AlbumCardViewModel
    let itemWidth: CGFloat
    let coverSize: CGFloat

    @State private var thumbnail: NSImage?

    private var cacheKey: String { "album-\(album.index)" }
    private var isSelected: Bool { state.selectedAlbumIndex == Int(album.index) }

    var body: some View {
        VStack(alignment: .leading, spacing: 5) {
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
                                .font(.system(size: coverSize * 0.25))
                                .foregroundColor(.white.opacity(0.3))
                        )
                }
            }
            .frame(width: coverSize, height: coverSize)
            .clipShape(RoundedRectangle(cornerRadius: 8))
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(isSelected ? Color.white : Color.clear, lineWidth: 2)
            )
            .shadow(color: .black.opacity(0.25), radius: 6, x: 0, y: 3)

            VStack(alignment: .leading, spacing: 2) {
                Text(album.name)
                    .font(.system(size: 13, weight: .medium))
                    .foregroundColor(.white)
                    .lineLimit(1)

                Text(album.artist)
                    .font(.system(size: 11))
                    .foregroundColor(.white.opacity(0.6))
                    .lineLimit(1)
            }
            .frame(width: coverSize, alignment: .leading)
        }
        .frame(width: itemWidth)
        .contentShape(Rectangle())
        .onTapGesture {
            state.selectAlbum(index: Int(album.index))
        }
        .task(id: cacheKey) {
            guard let engine = state.engine else { return }
            let maxPx = UInt32(max(16, Int(coverSize * 2)))
            let data = await engine.loadAlbumThumbnail(index: album.index, maxSize: maxPx)
            guard let data, !data.isEmpty else { return }
            thumbnail = await ThumbnailStore.shared.image(
                key: cacheKey,
                maxPixelSize: Int(maxPx),
                load: { data }
            )
        }
    }
}
