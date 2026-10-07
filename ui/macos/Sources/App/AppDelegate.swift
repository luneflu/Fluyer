import AppKit
import SwiftUI

/// Activates the unbundled SPM executable as a normal windowed app, owns the Now
/// Playing bridge and persists session state on quit.
///
/// ponytail: `swift run` produces a bare binary with no bundle, so macOS treats it
/// as a background process — no dock icon, no menu bar, and no key window until the
/// process is clicked in the Dock.
@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var state: AppState?
    private var nowPlaying: NowPlayingCoordinator?

    /// Called once the scene has built `AppState`; repeat calls are no-ops.
    func attach(_ state: AppState) {
        guard self.state == nil else { return }
        self.state = state
        nowPlaying = NowPlayingCoordinator(state: state)
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.regular)
        NSApp.activate(ignoringOtherApps: true)
        DispatchQueue.main.async {
            NSApp.windows.first?.makeKeyAndOrderFront(nil)
        }
    }

    func applicationWillTerminate(_ notification: Notification) {
        state?.saveSession()
    }

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        if !flag {
            sender.windows.first?.makeKeyAndOrderFront(self)
        }
        return true
    }
}
