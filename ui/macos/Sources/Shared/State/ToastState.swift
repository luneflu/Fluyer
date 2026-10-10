import Observation

/// Transient status messages from the core.
///
/// Split out of the former monolithic `AppState`.
@MainActor
@Observable
final class ToastState {
    private static let displayDuration: UInt64 = 3_000_000_000

    private(set) var message: String?

    /// ponytail: the previous implementation spawned an uncancellable `Task` per
    /// toast, so every message leaked a task that slept for three seconds. Cancelling
    /// on replace also stops a superseded toast from clearing its successor.
    @ObservationIgnored private var dismissal: Task<Void, Never>?

    func show(_ message: String) {
        self.message = message
        dismissal?.cancel()
        dismissal = Task { [weak self] in
            try? await Task.sleep(nanoseconds: Self.displayDuration)
            guard !Task.isCancelled else { return }
            self?.message = nil
        }
    }

    func dismiss() {
        dismissal?.cancel()
        dismissal = nil
        message = nil
    }
}
