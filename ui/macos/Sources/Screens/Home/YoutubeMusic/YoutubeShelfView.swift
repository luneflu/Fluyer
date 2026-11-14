import SwiftUI
import FluyerCore

/// Remote cover images for feed items: in-memory only, keyed by URL.
enum RemoteCover {
    private static let cache = NSCache<NSString, NSImage>()

    static func cached(_ url: String) -> NSImage? {
        cache.object(forKey: url as NSString)
    }

    static func load(_ url: String) async -> NSImage? {
        if let hit = cached(url) { return hit }
        guard let target = URL(string: url),
              let (data, _) = try? await URLSession.shared.data(from: target),
              let image = NSImage(data: data) else { return nil }
        cache.setObject(image, forKey: url as NSString)
        return image
    }
}

/// One home-feed shelf. Its renderer follows the item kind: tracks render as rows,
/// playlists/albums as cover cards sized with the local album carousel's `AlbumMetrics`.
struct YoutubeShelfView: View {
    let shelf: FeedShelf
    let playingVideoId: String?
    let onPlay: (SearchItem) -> Void

    @State private var height: CGFloat = 210

    /// Shelves are homogeneous (a track shelf or a playlist shelf), so one probe is enough.
    private var isTrackShelf: Bool { shelf.items.first?.kind == .track }

    var body: some View {
        VStack(alignment: .leading, spacing: Layout.YoutubeMusic.shelfTitleSpacing) {
            title
            if isTrackShelf {
                trackGrid
            } else {
                cards
            }
        }
    }

    private var title: some View {
        Text(shelf.title)
            .font(.system(size: 15, weight: .semibold))
            .foregroundColor(.white)
            .padding(.horizontal, AlbumMetrics.edge)
    }

    /// Same adaptive grid as the library's `MusicGridView`.
    private var trackGrid: some View {
        LazyVGrid(columns: trackColumns, spacing: Layout.MusicGrid.rowSpacing) {
            ForEach(Array(shelf.items.enumerated()), id: \.offset) { _, item in
                YoutubeTrackRow(item: item,
                                isPlaying: item.videoId != nil && item.videoId == playingVideoId) {
                    onPlay(item)
                }
            }
        }
        .padding(.horizontal, Layout.MusicGrid.horizontalPadding)
    }

    private let trackColumns = [
        GridItem(.adaptive(minimum: Layout.MusicGrid.minColumnWidth, maximum: .infinity),
                 spacing: Layout.MusicGrid.columnSpacing)
    ]

    private var cards: some View {
        GeometryReader { geo in
            let metrics = AlbumMetrics(width: geo.size.width, dpr: AlbumMetrics.screenScale)
            ScrollView(.horizontal, showsIndicators: false) {
                LazyHStack(spacing: AlbumMetrics.cardGap) {
                    ForEach(Array(shelf.items.enumerated()), id: \.offset) { _, item in
                        card(item, metrics: metrics)
                    }
                }
                .scrollTargetLayout()
            }
            .scrollTargetBehavior(.viewAligned)
            .padding(.horizontal, AlbumMetrics.edge)
            .onAppear { height = metrics.height }
            .onChange(of: metrics.height) { _, newHeight in height = newHeight }
        }
        .frame(height: height)
    }

    private func card(_ item: SearchItem, metrics: AlbumMetrics) -> some View {
        let isPlaying = item.videoId != nil && item.videoId == playingVideoId

        return CoverCard(
            coverSize: metrics.coverSize,
            title: item.title,
            subtitle: item.subtitle,
            isSelected: isPlaying,
            actions: item.videoId == nil ? [] : [("Play", { onPlay(item) })],
            onTap: item.videoId == nil ? nil : { onPlay(item) },
            cached: { _ in item.thumbnail.flatMap(RemoteCover.cached) },
            load: { _ in
                guard let url = item.thumbnail else { return nil }
                return await RemoteCover.load(url)
            }
        )
    }
}
