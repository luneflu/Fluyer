import AppKit

/// Wraps `NSOpenPanel` so the folder-picking configuration lives in one place.
enum FolderPicker {
    /// - Returns: the chosen directory paths, or an empty array if cancelled.
    @MainActor
    static func musicFolders(message: String) -> [String] {
        let panel = NSOpenPanel()
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.allowsMultipleSelection = true
        panel.prompt = "Scan Folder"
        panel.message = message

        guard panel.runModal() == .OK else { return [] }
        return panel.urls.map(\.path)
    }
}
