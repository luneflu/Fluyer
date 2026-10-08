import SwiftUI

/// Settings window (⌘,): library folders plus the persisted toggles.
struct SettingsView: View {
    @Bindable var state: AppState

    var body: some View {
        Form {
            Section("Music Folders") {
                if state.settings.musicFolders.isEmpty {
                    Text("No folders added")
                        .foregroundStyle(.secondary)
                }
                ForEach(state.settings.musicFolders, id: \.self) { folder in
                    HStack {
                        Text(folder)
                            .lineLimit(1)
                            .truncationMode(.middle)
                        Spacer()
                        Button("Remove") { state.removeFolder(folder) }
                    }
                }
                HStack {
                    Button("Add Folder…") { state.promptAddFolder() }
                    Button("Rescan") { state.scanSavedFolders() }
                        .disabled(state.settings.musicFolders.isEmpty || state.library.scanStatus.isScanning)
                }
            }

            Section("Appearance") {
                Toggle("Animated background", isOn: Bindable(state.settings).animatedBackground)
                Picker("Background source", selection: Bindable(state.settings).backdropSource) {
                    ForEach(BackdropSource.allCases, id: \.self) { Text($0.label).tag($0) }
                }
            }

            Section("Integrations") {
                Toggle("Discord Rich Presence", isOn: Binding(
                    get: { state.settings.discordRpc },
                    set: { state.setDiscordEnabled($0) }
                ))
            }
        }
        .formStyle(.grouped)
        .frame(width: 520)
        .frame(minHeight: 380)
    }
}
