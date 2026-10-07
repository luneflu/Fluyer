import FluyerCore
import Observation

/// Play-queue snapshot plus queue commands. Row `index` is the queue position, not
/// a library index. Only re-read while `isOpen`, so player-sync bursts don't
/// serialize the whole queue for a hidden panel.
@MainActor
@Observable
final class QueueState {
    private(set) var tracks: [TrackItemViewModel] = []

    var engine: FluyerAppEngine?

    var isOpen = false {
        didSet {
            if isOpen && !oldValue { reload() }
        }
    }

    func reload() {
        guard isOpen, let engine else { return }
        tracks = engine.getQueueView()
    }

    func goto(_ index: Int) {
        guard tracks.indices.contains(index) else { return }
        engine?.queueGoto(index: UInt64(index))
    }

    func remove(_ index: Int) {
        guard tracks.indices.contains(index) else { return }
        engine?.queueRemove(index: UInt64(index))
        reload()
    }

    /// Move by `delta` rows; no-op past either end.
    func move(_ index: Int, by delta: Int) {
        move(from: index, to: index + delta)
    }

    /// Move for a drop reported as a gap offset (SwiftUI `onMove`).
    func move(from index: Int, toOffset offset: Int) {
        move(from: index, to: Self.target(from: index, dropOffset: offset))
    }

    /// `offset` is the gap *before* removal, 0...count; dragging down lands one row
    /// earlier once the source is taken out.
    static func target(from index: Int, dropOffset offset: Int) -> Int {
        offset > index ? offset - 1 : offset
    }

    private func move(from index: Int, to target: Int) {
        guard index != target,
              tracks.indices.contains(index), tracks.indices.contains(target) else { return }
        engine?.queueMove(from: UInt64(index), to: UInt64(target))
        reload()
    }

    /// Stop playback and empty the queue.
    func clear() {
        guard !tracks.isEmpty else { return }
        engine?.queueClear()
        reload()
    }
}
