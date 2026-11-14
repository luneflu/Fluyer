/// Cache keys for `ThumbnailStore`.
///
/// ponytail: the requested pixel size is part of every key. The album carousel
/// (`"album-3"`, ~400pt) and the album's track rows (`"album-3"`, 88pt) used to
/// collide, so whichever loaded first won and the carousel rendered an 88px
/// bitmap stretched to 400pt.
enum ThumbnailKey {
    static func track(_ index: UInt64, px: Int) -> String {
        "track-\(index)@\(px)"
    }

    static func album(_ index: UInt64, px: Int) -> String {
        "album-\(index)@\(px)"
    }

    /// Queue covers are keyed by path (queue positions shift on reorder), not by index.
    static func queue(px: Int, path: String) -> String {
        "queue-\(px)-\(path)"
    }

    /// Current artwork is keyed by the playing track's path, not by its index,
    /// so a library rescan that renumbers tracks cannot serve the wrong cover.
    static func current(side: Int, path: String) -> String {
        "current-\(side)-\(path)"
    }

    /// Prefix covering every cached size of one cover, for invalidation.
    static func trackPrefix(_ index: UInt64) -> String {
        "track-\(index)@"
    }

    static func albumPrefix(_ index: UInt64) -> String {
        "album-\(index)@"
    }
}
