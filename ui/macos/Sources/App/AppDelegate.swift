import AppKit
import SwiftUI

/// Activates the unbundled SPM executable as a normal windowed app.
///
/// ponytail: `swift run` produces a bare binary with no bundle, so macOS treats it
/// as a background process — no dock icon, no menu bar, and no key window until the
/// process is clicked in the Dock.
final class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.regular)
        NSApp.activate(ignoringOtherApps: true)
        DispatchQueue.main.async {
            NSApp.windows.first?.makeKeyAndOrderFront(nil)
        }
    }

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        if !flag {
            sender.windows.first?.makeKeyAndOrderFront(self)
        }
        return true
    }
}
