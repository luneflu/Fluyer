import Foundation
import FluyerCore

/// Creates and owns the core engine, and owns the on-disk layout it needs.
///
/// The engine is created only once `attach(listener:)` is called, so the caller can
/// finish wiring up its observable state before the core starts emitting events.
@MainActor
final class EngineHandle {
    /// Bundle identifier, also used as the app-support and caches directory name.
    /// Hardcoded rather than read from `Bundle.main` because the app runs as a bare
    /// SPM executable in development, where there is no bundle identifier.
    static let identifier = "org.alvindimas05.fluyer"

    private(set) var engine: FluyerAppEngine?

    /// Starts the core. Any state objects the listener mutates must already exist.
    func attach(listener: FluyerEventListener) {
        guard engine == nil else { return }

        let fileManager = FileManager.default
        let dataDirectory = Self.directory(.applicationSupportDirectory)
        let cacheDirectory = Self.directory(.cachesDirectory)
        try? fileManager.createDirectory(at: dataDirectory, withIntermediateDirectories: true)
        try? fileManager.createDirectory(at: cacheDirectory, withIntermediateDirectories: true)

        do {
            engine = try FluyerAppEngine(
                dataDir: dataDirectory.path,
                cacheDir: cacheDirectory.path,
                listener: listener
            )
        } catch {
            NSLog("Fluyer core failed to start: \(error)")
        }
    }

    private static func directory(_ search: FileManager.SearchPathDirectory) -> URL {
        FileManager.default
            .urls(for: search, in: .userDomainMask)[0]
            .appendingPathComponent(identifier)
    }
}
