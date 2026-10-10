import SwiftUI

/// Fades out content an open sidebar covers, like `useAlbumList.shouldHide*Item`
/// in the legacy webview: an item hides once its right edge passes the sidebar's
/// left edge by more than `tolerance`. Faded items stop taking clicks.
enum SidebarOcclusion {
    static let coordinateSpace = "library"
    /// `extraToleranceWidth` in `useAlbumList.svelte.ts`.
    static let tolerance: CGFloat = 10

    static func isCovered(itemMaxX: CGFloat, sidebarMinX: CGFloat) -> Bool {
        itemMaxX > sidebarMinX + tolerance
    }
}

extension EnvironmentValues {
    /// Left edge of the open right sidebar in `SidebarOcclusion.coordinateSpace`;
    /// `.infinity` when closed.
    @Entry var sidebarMinX: CGFloat = .infinity
}

private struct SidebarOcclusionModifier: ViewModifier {
    @Environment(\.sidebarMinX) private var sidebarMinX
    @State private var maxX: CGFloat = 0

    func body(content: Content) -> some View {
        let covered = SidebarOcclusion.isCovered(itemMaxX: maxX, sidebarMinX: sidebarMinX)
        content
            .onGeometryChange(for: CGFloat.self) {
                $0.frame(in: .named(SidebarOcclusion.coordinateSpace)).maxX
            } action: { maxX = $0 }
            .opacity(covered ? 0 : 1)
            .allowsHitTesting(!covered)
            .accessibilityHidden(covered)
            .animation(.easeInOut(duration: 0.5), value: covered)
    }
}

extension View {
    func hiddenBySidebar() -> some View { modifier(SidebarOcclusionModifier()) }
}
