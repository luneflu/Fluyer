import Foundation
import SwiftUI
import AppKit
import FluyerCore

@Observable
public final class AppState: FluyerEventListener {
    public var playerBar: PlayerBarViewModel
    public var playView: PlayViewModel
    public var scanStatus: ScanStatusViewModel
    public var tracks: [TrackItemViewModel] = []
    public var albums: [AlbumCardViewModel] = []
    public var selectedAlbum: AlbumDetailViewModel? = nil
    public var selectedAlbumIndex: Int? = nil

    public var displayedTracks: [TrackItemViewModel] {
        if let _ = selectedAlbumIndex, let detail = selectedAlbum {
            return detail.tracks
        }
        return tracks
    }

    public var showPlayView: Bool = false
    public var toastMessage: String? = nil

    public private(set) var engine: FluyerAppEngine?

    public init() {
        let dummyPlayerBar = PlayerBarViewModel(
            trackIndex: -1,
            title: "No Track",
            artist: "",
            album: "",
            positionMs: 0,
            durationMs: 0,
            progressPct: 0.0,
            timeLabel: "0:00 / 0:00",
            isPlaying: false,
            repeatMode: .none,
            isShuffled: false,
            volume: 1.0
        )
        let dummyPlayView = PlayViewModel(
            track: nil,
            lyrics: [],
            currentLyricIndex: -1,
            palette: [ColorRgb(r: 28, g: 28, b: 36)]
        )
        let dummyScan = ScanStatusViewModel(
            isScanning: false,
            current: 0,
            total: 0,
            progressPct: 0.0,
            statusLabel: "Ready"
        )

        self.playerBar = dummyPlayerBar
        self.playView = dummyPlayView
        self.scanStatus = dummyScan

        setupEngine()
    }

    private func setupEngine() {
        let fm = FileManager.default
        let appSupport = fm.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
            .appendingPathComponent("org.alvindimas05.fluyer")
        let cacheDir = fm.urls(for: .cachesDirectory, in: .userDomainMask).first!
            .appendingPathComponent("org.alvindimas05.fluyer")

        try? fm.createDirectory(at: appSupport, withIntermediateDirectories: true)
        try? fm.createDirectory(at: cacheDir, withIntermediateDirectories: true)

        do {
            let engineInstance = try FluyerAppEngine(
                dataDir: appSupport.path,
                cacheDir: cacheDir.path,
                listener: self
            )
            self.engine = engineInstance
            refreshAll()
        } catch {
            print("Failed to initialize Fluyer core: \(error)")
        }
    }

    public func onEvent(event: FluyerEvent) {
        Task { @MainActor in
            self.handleEvent(event)
        }
    }

    @MainActor
    private func handleEvent(_ event: FluyerEvent) {
        switch event {
        case .playerBarUpdated(let vm):
            if let engine = engine {
                let fresh = engine.getPlayerBarView()
                if !fresh.title.isEmpty && fresh.title != "No Track" {
                    self.playerBar = fresh
                } else {
                    self.playerBar.isPlaying = vm.isPlaying
                    self.playerBar.positionMs = vm.positionMs
                    self.playerBar.durationMs = vm.durationMs
                    self.playerBar.progressPct = vm.progressPct
                    self.playerBar.timeLabel = vm.timeLabel
                    self.playerBar.repeatMode = vm.repeatMode
                    self.playerBar.isShuffled = vm.isShuffled
                }
                if showPlayView {
                    self.playView = engine.getPlayView()
                }
            }
            updateProgressTimer()

        case .playViewUpdated(let vm):
            self.playView = vm

        case .trackChanged(let track, _):
            ThumbnailStore.shared.invalidate("current-\(80)")
            ThumbnailStore.shared.invalidate("current-\(600)")
            if let engine = engine {
                self.playerBar = engine.getPlayerBarView()
                self.playView = engine.getPlayView()
                self.refreshTrackActiveStates()
            } else if let track = track {
                self.playerBar.title = track.title
                self.playerBar.artist = track.artist
                self.playerBar.album = track.album
            }
            updateProgressTimer()

        case .libraryUpdated:
            refreshAll()

        case .scanProgress(let vm):
            self.scanStatus = vm

        case .toast(let message):
            self.showToast(message)

        case .trackCoverLoaded(let index):
            ThumbnailStore.shared.invalidate("track-\(index)")
            if let engine = engine {
                self.playView = engine.getPlayView()
            }

        case .albumCoverLoaded(let index):
            ThumbnailStore.shared.invalidate("album-\(index)")

        case .lyricsLoaded:
            if let engine = engine {
                self.playView = engine.getPlayView()
            }
        }
    }

    public func refreshAll() {
        guard let engine = engine else { return }
        self.playerBar = engine.getPlayerBarView()
        self.playView = engine.getPlayView()
        self.scanStatus = engine.getScanStatus()

        let trackCount = engine.getTrackCount()
        var newTracks: [TrackItemViewModel] = []
        newTracks.reserveCapacity(Int(trackCount))
        for i in 0..<trackCount {
            if let t = engine.getTrackView(index: i) {
                newTracks.append(t)
            }
        }
        self.tracks = newTracks

        let albumCount = engine.getAlbumCount()
        var newAlbums: [AlbumCardViewModel] = []
        newAlbums.reserveCapacity(Int(albumCount))
        for i in 0..<albumCount {
            if let a = engine.getAlbumCard(index: i) {
                newAlbums.append(a)
            }
        }
        self.albums = newAlbums

        if let selected = selectedAlbum {
            self.selectedAlbum = engine.getAlbumDetail(index: selected.header.index)
        }
    }

    private func refreshTrackActiveStates() {
        guard let engine = engine else { return }
        let count = engine.getTrackCount()
        if tracks.count == Int(count) {
            for i in 0..<tracks.count {
                if let updated = engine.getTrackView(index: UInt64(i)) {
                    tracks[i].isCurrent = updated.isCurrent
                }
            }
        }
    }

    public func showToast(_ message: String) {
        self.toastMessage = message
        Task { @MainActor in
            try? await Task.sleep(nanoseconds: 3_000_000_000)
            if self.toastMessage == message {
                self.toastMessage = nil
            }
        }
    }

    // MARK: - Playback Commands

    private var progressTimer: Timer?

    private func updateProgressTimer() {
        if playerBar.isPlaying {
            if progressTimer == nil {
                progressTimer = Timer.scheduledTimer(withTimeInterval: 0.25, repeats: true) { [weak self] _ in
                    Task { @MainActor in
                        self?.tickProgress()
                    }
                }
            }
        } else {
            progressTimer?.invalidate()
            progressTimer = nil
        }
    }

    private func tickProgress() {
        guard let engine = engine, playerBar.isPlaying else { return }
        let pos = engine.getPosition()
        let dur = playerBar.durationMs
        playerBar.positionMs = pos
        if dur > 0 {
            playerBar.progressPct = Float(pos) / Float(dur)
            playerBar.timeLabel = "\(formatTime(pos)) / \(formatTime(dur))"
        }
        if showPlayView {
            playView.currentLyricIndex = engine.getActiveLyricIndex(positionMs: pos)
        }
    }

    public func togglePlay() {
        engine?.togglePlay()
        playerBar.isPlaying.toggle()
        updateProgressTimer()
    }

    public func next() {
        engine?.next()
    }

    public func previous() {
        engine?.previous()
    }

    public func seekPercent(_ pct: Float) {
        guard let engine = engine else { return }
        let clampedPct = pct.clamped(to: 0.0...1.0)
        let targetMs = UInt64(Float(playerBar.durationMs) * clampedPct)
        engine.seek(positionMs: targetMs)
        playerBar.positionMs = targetMs
        playerBar.progressPct = clampedPct
        playerBar.timeLabel = "\(formatTime(targetMs)) / \(formatTime(playerBar.durationMs))"
        if showPlayView {
            playView.currentLyricIndex = engine.getActiveLyricIndex(positionMs: targetMs)
        }
    }

    public func seek(positionMs: UInt64) {
        guard let engine = engine else { return }
        engine.seek(positionMs: positionMs)
        playerBar.positionMs = positionMs
        if playerBar.durationMs > 0 {
            playerBar.progressPct = Float(positionMs) / Float(playerBar.durationMs)
            playerBar.timeLabel = "\(formatTime(positionMs)) / \(formatTime(playerBar.durationMs))"
        }
        if showPlayView {
            playView.currentLyricIndex = engine.getActiveLyricIndex(positionMs: positionMs)
        }
    }

    private func formatTime(_ ms: UInt64) -> String {
        let totalSec = ms / 1000
        let min = totalSec / 60
        let sec = totalSec % 60
        return String(format: "%d:%02d", min, sec)
    }

    public func cycleRepeat() {
        engine?.cycleRepeat()
    }

    public func shuffle() {
        engine?.shuffle()
    }

    public func playTrack(at index: Int) {
        if let albumIndex = selectedAlbumIndex {
            engine?.playAlbumTrack(albumIndex: UInt64(albumIndex), trackIndex: UInt64(index))
        } else {
            engine?.playAllFromLibrary(startIndex: UInt64(index))
        }
    }

    public func selectAlbum(index: Int) {
        self.selectedAlbumIndex = index
        if let engine = engine {
            self.selectedAlbum = engine.getAlbumDetail(index: UInt64(index))
        }
    }

    public func clearAlbumSelection() {
        self.selectedAlbumIndex = nil
        self.selectedAlbum = nil
    }

    public func playSelectedAlbum() {
        if let idx = selectedAlbumIndex {
            playAlbum(at: UInt64(idx))
        }
    }

    public func queueSelectedAlbum() {
        if let idx = selectedAlbumIndex {
            queueAlbum(at: UInt64(idx))
        }
    }

    public func shuffleSelectedAlbum() {
        if let idx = selectedAlbumIndex {
            shuffleAlbum(at: UInt64(idx))
        }
    }

    public func playAlbum(at index: UInt64) {
        engine?.playAlbum(index: index)
    }

    public func playAlbumTrack(albumIndex: UInt64, trackIndex: UInt64) {
        engine?.playAlbumTrack(albumIndex: albumIndex, trackIndex: trackIndex)
    }

    public func queueAlbum(at index: UInt64) {
        engine?.queueAlbum(index: index)
    }

    public func shuffleAlbum(at index: UInt64) {
        engine?.shuffleAlbum(index: index)
    }

    // MARK: - Library Scanning

    public func promptAddFolder() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.allowsMultipleSelection = true
        panel.prompt = "Scan Folder"
        panel.message = "Choose music folders to add to Fluyer"

        if panel.runModal() == .OK {
            let paths = panel.urls.map { $0.path }
            if !paths.isEmpty {
                engine?.scanDirectories(directories: paths)
            }
        }
    }
}

fileprivate extension Comparable {
    func clamped(to limits: ClosedRange<Self>) -> Self {
        min(max(self, limits.lowerBound), limits.upperBound)
    }
}
