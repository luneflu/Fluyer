import SwiftUI
import AppKit
import Observation

/// Full screen: AppKit hosts the toolbar in its own `NSToolbarFullScreenWindow` whose
/// `NSVisualEffectView` material ignores `toolbarBackground(.hidden, for: .windowToolbar)`
/// (that only styles the windowed bar), so it is cleared here.
///
/// AppKit's reveal carries no usable signal: the window's `frame`, `alphaValue`,
/// `isVisible` and `occlusionState` stay put, and the effect views' alpha only flips
/// once per space animation. The pointer is polled instead: AppKit reveals the strip
/// when the pointer reaches the top edge, which is reported through
/// `FullScreenToolbarReveal.revealed`.
struct FullScreenToolbarClearer: NSViewRepresentable {
    func makeNSView(context: Context) -> NSView {
        context.coordinator.observe()
        // Strip whatever AppKit already built, so a window that opens full screen (or
        // reopens in a space) never shows the material first.
        MainActor.assumeIsolated {
            context.coordinator.patchNow()
            // The toolbar window can also exist before this view is built (a window
            // restored into full screen at launch never sends didEnterFullScreen).
            context.coordinator.patchSoon()
        }
        return NSView(frame: .zero)
    }

    func updateNSView(_ view: NSView, context: Context) {}

    func makeCoordinator() -> Coordinator { Coordinator() }

    final class Coordinator {
        private var tokens: [NSObjectProtocol] = []
        private var timer: Timer?
        private var monitor: Any?
        private var frameObservations: [NSKeyValueObservation] = []
        /// Rest position of each strip window: the largest `minY` it has shown. Learned
        /// instead of computed, so notches, scaled displays, external screens and
        /// animated menu bars cannot make the rest test wrong.
        private var restY: [ObjectIdentifier: CGFloat] = [:]
        /// Strip windows whose material is currently up, i.e. an out-animation is due.
        private var showing: Set<ObjectIdentifier> = []

        deinit {
            timer?.invalidate()
            if let monitor { NSEvent.removeMonitor(monitor) }
        }

        @MainActor
        func patchNow() {
            patch(revealed: FullScreenToolbarReveal.shared.revealed)
        }

        func observe() {
            let center = NotificationCenter.default
            tokens.append(center.addObserver(
                forName: NSWindow.willEnterFullScreenNotification, object: nil, queue: .main
            ) { _ in self.patchSoon() })
            tokens.append(center.addObserver(
                forName: NSWindow.didEnterFullScreenNotification, object: nil, queue: .main
            ) { _ in
                self.patchSoon()
                self.startPolling()
            })
            // A strip that moved to another display needs its rest position relearned.
            tokens.append(center.addObserver(
                forName: NSWindow.didChangeScreenNotification, object: nil, queue: .main
            ) { note in
                guard let window = note.object as? NSWindow else { return }
                self.restY[ObjectIdentifier(window)] = nil
            })
            tokens.append(center.addObserver(
                forName: NSWindow.didExitFullScreenNotification, object: nil, queue: .main
            ) { _ in
                MainActor.assumeIsolated {
                    self.timer?.invalidate()
                    self.timer = nil
                    if let monitor = self.monitor { NSEvent.removeMonitor(monitor); self.monitor = nil }
                    self.frameObservations.forEach { $0.invalidate() }
                    self.frameObservations.removeAll()
                    FullScreenToolbarReveal.shared.set(false)
                }
            })
        }

        /// AppKit's own reveal threshold is the very top edge.
        @MainActor
        private func updateReveal() {
            guard let screen = NSScreen.main else { return }
            let revealed = NSEvent.mouseLocation.y >= screen.frame.maxY - 4
            FullScreenToolbarReveal.shared.set(revealed)
            patch(revealed: revealed)
        }

        @MainActor
        private func startPolling() {
            // Pointer moves drive the reveal: polling left the material up for up to
            // 100 ms after the pointer left, which read as a shadow flash.
            if monitor == nil {
                monitor = NSEvent.addLocalMonitorForEvents(matching: [.mouseMoved, .mouseExited]) { event in
                    MainActor.assumeIsolated { self.updateReveal() }
                    return event
                }
            }
            timer?.invalidate()
            timer = Timer.scheduledTimer(withTimeInterval: 1.0 / 60.0, repeats: true) { _ in
                MainActor.assumeIsolated { self.updateReveal() }
            }
        }

        /// The full-screen window is built during the space animation, so retry.
        func patchSoon() {
            for delay in [0.0, 0.1, 0.2, 0.35, 0.6, 1.0] {
                DispatchQueue.main.asyncAfter(deadline: .now() + delay) {
                    self.patch(revealed: FullScreenToolbarReveal.shared.revealed)
                }
            }
        }

        private func patch(revealed: Bool) {
            for window in NSApp.windows
            where String(describing: type(of: window)).contains("ToolbarFullScreen") {
                window.isOpaque = false
                window.backgroundColor = .clear
                // AppKit's out-animation ends when the strip is back at rest
                // (`frame.minY` = screen top - height; it drops 24pt while revealed).
                // The material stays up until then, so the shadow AppKit composites on
                // the way out lands under the grey rather than over the backdrop.
                if frameObservations.isEmpty {
                    frameObservations.append(window.observe(\.frame, options: [.new]) { window, _ in
                        DispatchQueue.main.async {
                            guard Self.atRest(window, restY: &self.restY) else { return }
                            self.showing.remove(ObjectIdentifier(window))
                            Self.strip(window.contentView, revealed: false)
                        }
                    })
                }
                if revealed {
                    showing.insert(ObjectIdentifier(window))
                    Self.strip(window.contentView, revealed: true)
                } else if !revealed {
                    // Nothing was revealed (entry, restore, second space), so no
                    // out-animation to wait for: clear now.
                    if !showing.contains(ObjectIdentifier(window)) {
                        Self.strip(window.contentView, revealed: false)
                    } else if Self.atRest(window, restY: &restY) {
                        showing.remove(ObjectIdentifier(window))
                        Self.strip(window.contentView, revealed: false)
                    }
                }
            }
        }

        /// True once the strip has slid back to the lowest point it ever reaches, which
        /// is its resting place in full screen.
        private static func atRest(_ window: NSWindow, restY: inout [ObjectIdentifier: CGFloat]) -> Bool {
            let id = ObjectIdentifier(window)
            let y = window.frame.minY
            let known = restY[id] ?? -.greatestFiniteMagnitude
            if y > known + 0.5 {
                restY[id] = y
                return false
            }
            return y >= known - 0.5
        }

        private static func strip(_ view: NSView?, revealed: Bool) {
            guard let view else { return }
            if let effect = view as? NSVisualEffectView {
                effect.state = revealed ? .active : .inactive
                effect.alphaValue = revealed ? 1 : 0
            }
            // The strip's shadow can also live on the subviews' layers.
            if view.wantsLayer, let layer = view.layer {
                layer.shadowOpacity = 0
                layer.shadowRadius = 0
            }
            view.subviews.forEach { strip($0, revealed: revealed) }
        }
    }
}

/// True while the full-screen toolbar strip is up (pointer at the top edge), i.e. while
/// the bar is fixed over the content instead of floating over it.
@MainActor
@Observable
final class FullScreenToolbarReveal {
    static let shared = FullScreenToolbarReveal()
    private(set) var revealed = false

    func set(_ value: Bool) {
        if revealed != value { revealed = value }
    }
}
