import AppKit
import FluyerCore

/// Cover art for every view: track, album and now-playing thumbnails plus the
/// backdrop input. The only Swift code that asks the engine for images; views call
/// these and never touch the engine. Decoded bitmaps are cached in `ThumbnailStore`.
@MainActor
final class CoverState {
    var engine: FluyerAppEngine?

    private let store = ThumbnailStore.shared

    /// Already-decoded bitmap for `key`, so a reused row can draw without waiting.
    func cached(_ key: String) -> NSImage? {
        store.peek(key)
    }

    func track(_ index: UInt64, px: Int) async -> NSImage? {
        guard let engine else { return nil }
        return await store.image(key: ThumbnailKey.track(index, px: px)) {
            await engine.artworkLoadTrackThumbnail(index: index, maxSize: UInt32(px))
        }
    }

    func album(_ index: UInt64, px: Int) async -> NSImage? {
        guard let engine else { return nil }
        return await store.image(key: ThumbnailKey.album(index, px: px)) {
            await engine.artworkLoadAlbumThumbnail(index: index, maxSize: UInt32(px))
        }
    }

    /// Playing track's cover. `path` is part of the key so a track change never
    /// serves the previous song's bitmap.
    func current(px: Int, path: String) async -> NSImage? {
        guard let engine else { return nil }
        return await store.image(key: ThumbnailKey.current(side: px, path: path)) {
            await engine.artworkLoadCurrentThumbnail(maxSize: UInt32(px))
        }
    }

    /// Playing track's cover without caching, for one-off large uses
    /// (backdrop, Now Playing widget) that would only evict grid thumbnails.
    func currentUncached(px: Int) async -> NSImage? {
        guard let engine else { return nil }
        return await engine.artworkLoadCurrentThumbnail(maxSize: UInt32(px)).flatMap { NSImage(data: $0) }
    }

    /// Backdrop input: the cover in `.artwork` mode, otherwise (or with no cover)
    /// Rust's palette / grey block square.
    func backdrop(source: BackdropSource, hasTrack: Bool) async -> NSImage? {
        guard let engine else { return nil }
        if source == .artwork, hasTrack, let cover = await currentUncached(px: Self.backdropPixels) {
            return cover
        }
        return Self.image(from: await engine.backdropLoadBlockArtwork())
    }

    func invalidateTrack(_ index: UInt64) {
        store.invalidate(prefix: ThumbnailKey.trackPrefix(index))
    }

    func invalidateAlbum(_ index: UInt64) {
        store.invalidate(prefix: ThumbnailKey.albumPrefix(index))
    }

    /// ponytail: the backdrop is full-screen and blurred to mush, so 1200px is ample
    /// and keeps the decode off the critical path.
    private static let backdropPixels = 1200

    private static func image(from frame: AnimatedBackgroundFrame) -> NSImage? {
        let w = Int(frame.width), h = Int(frame.height)
        guard w > 0, h > 0, frame.rgba.count == w * h * 4,
              let provider = CGDataProvider(data: Data(frame.rgba) as CFData),
              let cg = CGImage(width: w, height: h, bitsPerComponent: 8, bitsPerPixel: 32, bytesPerRow: w * 4,
                               space: CGColorSpace(name: CGColorSpace.sRGB)!,
                               bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.noneSkipLast.rawValue),
                               provider: provider, decode: nil, shouldInterpolate: true, intent: .defaultIntent)
        else { return nil }
        return NSImage(cgImage: cg, size: NSSize(width: w, height: h))
    }
}
