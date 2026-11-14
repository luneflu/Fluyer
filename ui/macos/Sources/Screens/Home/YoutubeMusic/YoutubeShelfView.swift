import SwiftUI
import FluyerCore

/// Remote cover images for feed items: decoded once per (URL, display size) pair.
///
/// The API hands out ~1600px art; decoding that per card costs ~9 MB of bitmap each, so
/// the cache is size-keyed, downsampled on decode, and budgeted (cost = pixels×4).
enum RemoteCover {
    private static let cache: NSCache<NSString, NSImage> = {
        let cache = NSCache<NSString, NSImage>()
        cache.countLimit = 80
        cache.totalCostLimit = 64 * 1024 * 1024
        return cache
    }()

    /// Drops every decoded feed cover (tab switch, so the library's bitmaps get the room).
    static func clear() {
        cache.removeAllObjects()
    }

    static func cached(_ url: String, px: Int) -> NSImage? {
        cache.object(forKey: key(url, px) as NSString)
    }

    static func load(_ url: String, px: Int) async -> NSImage? {
        if let hit = cached(url, px: px) { return hit }
        // Only the play view needs the full-size art the core hands out; cards ask the
        // CDN for their own pixel size instead of downloading ~1600px.
        guard let target = URL(string: sized(url, px: px)),
              let (data, _) = try? await URLSession.shared.data(from: target),
              let image = downsample(data, px: px) else { return nil }
        cache.setObject(image, forKey: key(url, px) as NSString, cost: px * px * 4)
        return image
    }

    /// Rewrites the ggpht size token (`=w1600-h1600-l90-rj`, `=s576`) to `px`.
    static func sized(_ url: String, px: Int) -> String {
        guard let eq = url.lastIndex(of: "=") else { return url }
        let suffix = url[url.index(after: eq)...]
        let tail: Substring
        if suffix.hasPrefix("s") {
            // `=s576` (no tail) and `=s192-c-k-…` both work: the tail may be empty.
            let body = suffix.dropFirst()
            let end = body.firstIndex(where: { !$0.isNumber }) ?? body.endIndex
            tail = body[end...]
        } else if suffix.hasPrefix("w") {
            guard var cursor = suffix.dropFirst().firstIndex(where: { !$0.isNumber }),
                  suffix[cursor...].hasPrefix("-h") else { return url }
            cursor = suffix.index(cursor, offsetBy: 2)
            let body = suffix[cursor...]
            let end = body.firstIndex(where: { !$0.isNumber }) ?? body.endIndex
            tail = body[end...]
        } else {
            return url
        }
        let size = max(16, px)
        return String(url[...eq]) + "w\(size)-h\(size)" + tail
    }

    private static func key(_ url: String, _ px: Int) -> String {
        "\(url)@\(px)"
    }

    /// ImageIO thumbnailing: decodes straight to `px`, so the full-size bitmap never
    /// exists in memory.
    private static func downsample(_ data: Data, px: Int) -> NSImage? {
        guard let source = CGImageSourceCreateWithData(data as CFData, nil) else { return nil }
        let options: [CFString: Any] = [
            kCGImageSourceCreateThumbnailFromImageAlways: true,
            kCGImageSourceCreateThumbnailWithTransform: true,
            kCGImageSourceThumbnailMaxPixelSize: px,
        ]
        guard let cgImage = CGImageSourceCreateThumbnailAtIndex(source, 0, options as CFDictionary) else { return nil }
        return NSImage(cgImage: cgImage, size: NSSize(width: px, height: px))
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
            cached: { px in item.thumbnail.flatMap { RemoteCover.cached($0, px: px) } },
            load: { px in
                guard let url = item.thumbnail else { return nil }
                return await RemoteCover.load(url, px: px)
            }
        )
    }
}
