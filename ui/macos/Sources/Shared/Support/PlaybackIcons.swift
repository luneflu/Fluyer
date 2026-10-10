import FluyerCore

/// SF Symbol names driven by transport state, shared by the player bar and the
/// now-playing control card.
enum PlaybackIcons {
    static func repeatIcon(_ mode: NativeRepeatMode) -> String {
        switch mode {
        case .one: return "repeat.1"
        case .all, .none: return "repeat"
        }
    }

    /// - Parameter level: current volume, nominally 0...1.
    static func volume(_ level: Float) -> String {
        if level <= 0.001 { return "speaker.slash.fill" }
        if level < 0.33 { return "speaker.wave.1.fill" }
        if level < 0.66 { return "speaker.wave.2.fill" }
        return "speaker.wave.3.fill"
    }
}
