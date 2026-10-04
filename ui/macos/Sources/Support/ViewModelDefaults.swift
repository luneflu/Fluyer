import FluyerCore

/// Placeholder view models for the window that renders before the core has
/// reported anything (and if it never starts at all).
///
/// ponytail: these mirror the Rust constructors `PlayerBarViewModel::default`,
/// `PlayViewModel::empty` and `ScanStatusViewModel::idle` exactly. Anything
/// looser shows a stray "Unknown Title" or a blank scan label on launch.
extension PlayerBarViewModel {
    static let noTrack = PlayerBarViewModel(
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
}

extension PlayViewModel {
    static let empty = PlayViewModel(
        track: nil,
        lyrics: [],
        currentLyricIndex: -1,
        palette: [ColorRgb(r: 28, g: 28, b: 36)]
    )
}

extension ScanStatusViewModel {
    static let idle = ScanStatusViewModel(
        isScanning: false,
        current: 0,
        total: 0,
        progressPct: 0.0,
        statusLabel: "Ready"
    )
}
