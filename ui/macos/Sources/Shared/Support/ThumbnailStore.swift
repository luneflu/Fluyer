import AppKit

/// Bounded LRU of decoded thumbnails.
///
/// ponytail: an earlier revision had no cache at all, which made scrolling
/// re-run a full FFI + decode pass for every visible cell on every frame.
/// This keeps only small, pre-downscaled bitmaps (never full-res covers) and
/// caps both count and bytes so it cannot grow into a memory regression.
@MainActor
final class ThumbnailStore {
    static let shared = ThumbnailStore()

    private let cache = NSCache<NSString, NSImage>()

    /// `NSCache` cannot enumerate its keys, so mirror what we inserted to keep
    /// prefix invalidation working after it has evicted on its own.
    @ObservationIgnored private var insertedKeys: Set<String> = []

    init(countLimit: Int = 300, byteLimit: Int = 24 * 1024 * 1024) {
        cache.countLimit = countLimit
        cache.totalCostLimit = byteLimit
    }

    /// Synchronous peek so a row can render its bitmap immediately on reuse.
    func peek(_ key: String) -> NSImage? {
        cache.object(forKey: key as NSString)
    }

    /// Drop every cached size of one cover.
    func invalidate(prefix: String) {
        let doomed = insertedKeys.filter { $0.hasPrefix(prefix) }
        for key in doomed {
            insertedKeys.remove(key)
            cache.removeObject(forKey: key as NSString)
        }
    }

    /// Fetch already-downscaled JPEG bytes from the core, then decode off the
    /// main actor. `load` is the async UniFFI call; decode happens on a detached
    /// task so neither blocks SwiftUI's render loop.
    ///
    /// Returns the cached bitmap without calling `load` when the key is warm.
    func image(key: String, load: @escaping () async -> Data?) async -> NSImage? {
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

    private func store(_ image: NSImage, for key: String) {
        let width = image.size.width > 0 ? image.size.width : 1
        let height = image.size.height > 0 ? image.size.height : 1
        cache.setObject(image, forKey: key as NSString, cost: Int(width * height * 4))
        insertedKeys.insert(key)
    }

    private nonisolated static func decode(_ data: Data) async -> NSImage? {
        await Task.detached(priority: .userInitiated) {
            NSImage(data: data)
        }.value
    }
}
