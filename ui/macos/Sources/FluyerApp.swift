import SwiftUI

@main
struct FluyerApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @State private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            ContentView(state: appState)
                .preferredColorScheme(.dark)
                // App-wide accent: now-playing marks, toggles, prominent buttons.
                .tint(.white)
                .onAppear { appDelegate.attach(appState) }
        }
        .windowStyle(.titleBar)
        .windowToolbarStyle(.unified(showsTitle: true))
        .defaultSize(width: 1050, height: 720)
        // ponytail: Windows' left "menu" sidebar maps to the native menu bar here.
        .commands {
            CommandGroup(after: .newItem) {
                Button("Open Music Folder…") { appState.promptAddFolder() }
                    .keyboardShortcut("o")
            }
            CommandMenu("Playback") {
                Button("Play All") { appState.playAll() }
                    .disabled(appState.library.tracks.isEmpty)
                Button(appState.playback.bar.isPlaying ? "Pause" : "Play") { appState.playback.togglePlay() }
                Button("Next") { appState.playback.next() }
                    .keyboardShortcut(.rightArrow)
                Button("Previous") { appState.playback.previous() }
                    .keyboardShortcut(.leftArrow)
                Divider()
                Button("Show Play Screen") { appState.playback.showPlayView = true }
                    .keyboardShortcut("p", modifiers: [.command, .shift])
            }
            CommandGroup(after: .sidebar) {
                Button(appState.queue.isOpen ? "Hide Queue" : "Show Queue") { appState.queue.isOpen.toggle() }
                    .keyboardShortcut("l")
            }
        }

        Settings {
            SettingsView(state: appState)
                .preferredColorScheme(.dark)
                .tint(.white)
        }
    }
}
