import Foundation
import FluyerCore
import Observation

/// What the animated backdrop renders: the real cover, or Rust's palette-block square.
enum BackdropSource: String, Codable, CaseIterable {
    case artwork, blocks

    var label: String {
        switch self {
        case .artwork: "Artwork"
        case .blocks: "Color blocks"
        }
    }
}

/// User settings for the window. The core owns the file (`<data dir>/settings.json`)
/// and every rule that used to live here: JSON shape, folder normalization, dedupe,
/// volume clamp, unknown backdrop value. This type is the observable mirror views bind
/// to; it never touches disk.
@MainActor
@Observable
final class SettingsState {
    private(set) var musicFolders: [String] = []
    var animatedBackground = true { didSet { if animatedBackground != oldValue { save() } } }
    var backdropSource = BackdropSource.artwork { didSet { if backdropSource != oldValue { save() } } }
    var discordRpc = true { didSet { if discordRpc != oldValue { save() } } }
    /// Last session volume; written at shutdown, not per slider tick. Callers pass
    /// `PlaybackState`'s already-clamped volume.
    var volume: Float = 1.0 { didSet { if volume != oldValue { save() } } }

    /// Set once by `AppState` right after the core starts.
    @ObservationIgnored var engine: FluyerAppEngine?

    /// True while applying values that came back from the core, so they don't save again.
    @ObservationIgnored private var isLoading = false

    /// Reads the stored settings. Called after `engine` is assigned; a nil engine keeps
    /// the defaults (previews, tests).
    func load() {
        guard let engine else { return }
        apply(engine.settingsGet())
    }

    /// Adds folders not already present. The core normalizes and dedupes; the return
    /// value is what actually got stored.
    @discardableResult
    func addFolders(_ paths: [String]) -> [String] {
        let before = musicFolders
        musicFolders += paths
        save()
        return musicFolders.filter { !before.contains($0) }
    }

    /// Forgets a folder; returns the stored spelling (what scans used), or nil if unknown.
    @discardableResult
    func removeFolder(_ path: String) -> String? {
        let key = Self.matchingForm(path)
        guard let row = musicFolders.firstIndex(where: { Self.matchingForm($0) == key }) else { return nil }
        let stored = musicFolders.remove(at: row)
        save()
        return stored
    }

    /// Compares a picked path with a stored one: same trailing-slash trim the core uses.
    private static func matchingForm(_ path: String) -> String {
        let trimmed = path.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmed.count > 1 else { return trimmed }
        let stripped = String(trimmed.reversed().drop(while: { $0 == "/" }).reversed())
        return stripped.isEmpty ? "/" : stripped
    }

    private func save() {
        guard !isLoading, let engine else { return }
        apply(engine.settingsSet(settings: SettingsViewModel(
            musicFolders: musicFolders,
            animatedBackground: animatedBackground,
            backdropSource: backdropSource.rawValue,
            discordRpc: discordRpc,
            volume: volume
        )))
    }

    /// Mirrors the core's canonical values back into the observable properties.
    private func apply(_ stored: SettingsViewModel) {
        isLoading = true
        defer { isLoading = false }
        musicFolders = stored.musicFolders
        animatedBackground = stored.animatedBackground
        backdropSource = BackdropSource(rawValue: stored.backdropSource) ?? .artwork
        discordRpc = stored.discordRpc
        volume = stored.volume
    }
}
