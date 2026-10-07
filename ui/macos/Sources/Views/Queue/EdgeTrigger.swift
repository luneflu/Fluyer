import CoreGraphics

/// Hover rules for the right-edge queue sidebar, ported from `Sidebar.svelte`.
/// Points are in the library area's coordinates (toolbar to player bar).
struct EdgeTrigger {
    /// The cursor counts as "on the edge" within this many points of the right side.
    /// `Sidebar.svelte` uses 1px; a fast flick can skip that, so leaving the area
    /// through this band counts too.
    static let edgeBand: CGFloat = 2
    /// While open, the cursor near the window edge keeps it open (`nearScreenEdge`).
    static let keepOpenBand: CGFloat = 20
    /// Resting time on the edge before opening.
    static let openDelay: Duration = .milliseconds(150)

    enum Action: Equatable { case none, armOpen, cancelOpen, close }

    let size: CGSize
    /// Left edge of the open panel's hit area (`sidebarLeft`).
    let panelMinX: CGFloat

    func isOnEdge(_ p: CGPoint) -> Bool {
        p.x >= size.width - Self.edgeBand && p.y >= 0 && p.y <= size.height
    }

    /// Decide for a hover sample. `nil` means the cursor left the area; `last` is
    /// where it was seen before that.
    func action(at point: CGPoint?, last: CGPoint?, isOpen: Bool) -> Action {
        guard let p = point else {
            if isOpen { return .none }
            // Left through the right edge between moves (`onBodyMouseLeave`).
            if let last, last.x >= size.width - Self.keepOpenBand, last.y >= 0, last.y <= size.height {
                return .armOpen
            }
            return .cancelOpen
        }
        if !isOpen { return isOnEdge(p) ? .armOpen : .cancelOpen }
        let inside = p.x >= panelMinX && p.y >= 0 && p.y <= size.height
        let nearEdge = p.x >= size.width - Self.keepOpenBand
        return inside || nearEdge ? .none : .close
    }
}
