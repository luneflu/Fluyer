import AppKit

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

/// Resolved card geometry for the current window size.
///
/// The rule's ratio splits `width - 2·edge + gap` into whole slots, so n covers,
/// n−1 gaps and both edges fill the width exactly (Windows `ItemWidth` does the
/// same). Scrolling snaps to cards, and the scroll range is a whole number of
/// slots, so the last position lines up as cleanly as the first.
struct AlbumMetrics: Equatable {
    static let cardGap = Layout.Albums.cardGap
    static let edge = Layout.Albums.edge
    static let cardLabelHeight = Layout.Albums.cardLabelHeight

    /// Cover plus one gap.
    let itemWidth: CGFloat
    let coverSize: CGFloat
    let height: CGFloat
    /// Cards per row in the album grid (`Math.round(1 / widthRatio)`).
    let columnCount: Int

    init(width: CGFloat, dpr: Double) {
        let itemWidth = Self.itemWidth(width: width, dpr: dpr)
        let coverSize = max(16, itemWidth - Self.cardGap)
        self.itemWidth = itemWidth
        self.coverSize = coverSize
        self.height = coverSize + Self.cardLabelHeight
        self.columnCount = max(1, Int((max(1, width - Self.edge * 2 + Self.cardGap) / itemWidth).rounded()))
    }

    static func itemWidth(width: CGFloat, dpr: Double) -> CGFloat {
        let available = max(1, width - Self.edge * 2 + Self.cardGap)
        for rule in kResponsiveRules where width >= rule.minWidth && dpr >= rule.minDpr {
            return rule.widthRatio * available
        }
        return 0.5 * available
    }

    static var screenScale: Double {
        Double(NSScreen.main?.backingScaleFactor ?? 2.0)
    }

    /// Sidebar width for a window `width`: two album slots minus the 12pt outer
    /// padding on each side (`Sidebar.svelte`: `sidebarStore.width - 24`, where
    /// `sidebarStore.width = itemWidth * 2` in `useAlbumList`).
    static func sidebarWidth(forWidth width: CGFloat) -> CGFloat {
        max(1, itemWidth(width: width, dpr: screenScale) * 2 - Layout.Queue.outerInset * 2)
    }
}
