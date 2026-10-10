import Foundation
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

/// User settings persisted as JSON in Application Support. `url == nil` keeps
/// everything in memory (tests, headless).
@MainActor
@Observable
final class SettingsState {
    static let fileName = "settings.json"

    private struct Snapshot: Codable {
        var musicFolders: [String] = []
        var animatedBackground = true
        var backdropSource = BackdropSource.artwork
        var discordRpc = true
        var volume: Float = 1.0

        init() {}

        // Tolerate older/partial files: every missing key keeps its default.
        init(from decoder: Decoder) throws {
            let c = try decoder.container(keyedBy: CodingKeys.self)
            musicFolders = try c.decodeIfPresent([String].self, forKey: .musicFolders) ?? []
            animatedBackground = try c.decodeIfPresent(Bool.self, forKey: .animatedBackground) ?? true
            // Unknown raw value (newer build wrote it) degrades to default, not a reset of all settings.
            backdropSource = (try? c.decodeIfPresent(BackdropSource.self, forKey: .backdropSource)) ?? .artwork
            discordRpc = try c.decodeIfPresent(Bool.self, forKey: .discordRpc) ?? true
            volume = try c.decodeIfPresent(Float.self, forKey: .volume) ?? 1.0
        }
    }

    @ObservationIgnored private let url: URL?
    @ObservationIgnored private var isLoading = false

    private(set) var musicFolders: [String] = []
    var animatedBackground = true { didSet { if animatedBackground != oldValue { save() } } }
    var backdropSource = BackdropSource.artwork { didSet { if backdropSource != oldValue { save() } } }
    var discordRpc = true { didSet { if discordRpc != oldValue { save() } } }
    /// Last session volume; written at shutdown, not per slider tick. Callers pass
    /// `PlaybackState`'s already-clamped volume.
    var volume: Float = 1.0 { didSet { if volume != oldValue { save() } } }

    init(url: URL? = nil) {
        self.url = url
        load()
    }

    /// Adds folders not already present (trailing slash ignored). Returns the ones added.
    @discardableResult
    func addFolders(_ paths: [String]) -> [String] {
        var added: [String] = []
        for path in paths.map(Self.normalize) where !path.isEmpty && !musicFolders.contains(path) {
            musicFolders.append(path)
            added.append(path)
        }
        if !added.isEmpty { save() }
        return added
    }

    /// Forgets a folder; returns the stored spelling (what scans used), or nil if unknown.
    @discardableResult
    func removeFolder(_ path: String) -> String? {
        let key = Self.normalize(path)
        guard let row = musicFolders.firstIndex(of: key) else { return nil }
        musicFolders.remove(at: row)
        save()
        return key
    }

    static func normalize(_ path: String) -> String {
        let trimmed = path.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmed.count > 1, trimmed.hasSuffix("/") else { return trimmed }
        return String(trimmed.dropLast(trimmed.reversed().prefix(while: { $0 == "/" }).count))
    }

    private func load() {
        guard let url, let data = try? Data(contentsOf: url) else { return }
        isLoading = true
        defer { isLoading = false }
        do {
            let snap = try JSONDecoder().decode(Snapshot.self, from: data)
            var seen = Set<String>()
            musicFolders = snap.musicFolders.map(Self.normalize).filter { !$0.isEmpty && seen.insert($0).inserted }
            animatedBackground = snap.animatedBackground
            backdropSource = snap.backdropSource
            discordRpc = snap.discordRpc
            volume = snap.volume.clamped(to: 0...1)
        } catch {
            // Unreadable file: keep a copy so the next save doesn't destroy it.
            NSLog("Settings unreadable, using defaults: \(error)")
            let bad = url.appendingPathExtension("bad")
            try? FileManager.default.removeItem(at: bad)
            try? FileManager.default.copyItem(at: url, to: bad)
        }
    }

    private func save() {
        guard let url, !isLoading else { return }
        var snap = Snapshot()
        snap.musicFolders = musicFolders
        snap.animatedBackground = animatedBackground
        snap.backdropSource = backdropSource
        snap.discordRpc = discordRpc
        snap.volume = volume
        do {
            try FileManager.default.createDirectory(
                at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
            let encoder = JSONEncoder()
            encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
            try encoder.encode(snap).write(to: url, options: .atomic)
        } catch {
            NSLog("Settings save failed: \(error)")
        }
    }
}
