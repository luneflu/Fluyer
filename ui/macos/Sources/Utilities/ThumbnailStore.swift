import AppKit
import SwiftUI
import ImageIO

/// Bounded LRU of decoded thumbnails.
///
/// ponytail: an earlier revision had no cache at all, which made scrolling
/// re-run a full FFI + decode pass for every visible cell on every frame.
/// This keeps only small, pre-downscaled bitmaps (never full-res covers) and
/// caps both count and bytes so it cannot grow into a memory regression.
@MainActor
public final class ThumbnailStore {
    public static let shared = ThumbnailStore()

    private let cache = NSCache<NSString, NSImage>()

    public init(countLimit: Int = 300, byteLimit: Int = 24 * 1024 * 1024) {
        cache.countLimit = countLimit
        cache.totalCostLimit = byteLimit
    }

    /// Synchronous peek so a row can render its bitmap immediately on reuse.
    public func peek(_ key: String) -> NSImage? {
        cache.object(forKey: key as NSString)
    }

    public func invalidate(_ key: String) {
        cache.removeObject(forKey: key as NSString)
    }

    public func clear() {
        cache.removeAllObjects()
    }

    private func store(_ image: NSImage, for key: String) {
        let w = image.size.width > 0 ? image.size.width : 1
        let h = image.size.height > 0 ? image.size.height : 1
        cache.setObject(image, forKey: key as NSString, cost: Int(w * h * 4))
    }
}

// MARK: - Async loading

extension ThumbnailStore {
    /// Fetches already-downscaled JPEG bytes from the core, then decodes off the
    /// main actor. `load` is the async UniFFI call; decode happens on a detached
    /// task so neither blocks SwiftUI's render loop.
    public func image(
        key: String,
        maxPixelSize: Int,
        load: @escaping () async -> Data?
    ) async -> NSImage? {
        if let hit = peek(key) {
            return hit
        }
        guard let data = await load(), !data.isEmpty else {
            return nil
        }
        guard let image = await Self.decode(data) else {
            return nil
        }
        store(image, for: key)
        return image
    }

    private nonisolated static func decode(_ data: Data) async -> NSImage? {
        await Task.detached(priority: .userInitiated) {
            NSImage(data: data)
        }.value
    }
}

// MARK: - Fallback helper

public enum ImageUtils {
    /// Last-resort decode for callers that already hold raw image bytes.
    public static func thumbnail(from data: Data?, maxPixelSize: Int = 160) -> NSImage? {
        guard let data = data as CFData?,
              let source = CGImageSourceCreateWithData(data, nil) else {
            return nil
        }
        let options: [CFString: Any] = [
            kCGImageSourceCreateThumbnailFromImageAlways: true,
            kCGImageSourceCreateThumbnailWithTransform: true,
            kCGImageSourceThumbnailMaxPixelSize: maxPixelSize,
            kCGImageSourceShouldCacheImmediately: true
        ]
        guard let cgThumb = CGImageSourceCreateThumbnailAtIndex(source, 0, options as CFDictionary) else {
            return nil
        }
        return NSImage(
            cgImage: cgThumb,
            size: NSSize(width: cgThumb.width / 2, height: cgThumb.height / 2)
        )
    }
}
