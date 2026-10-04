import Foundation

/// Elapsed/duration formatting for the player bar, the now-playing card and the
/// core's own `time_label`.
///
/// ponytail: the core already formats `PlayerBarViewModel::time_label` (see
/// `view_models::format_time`), but the UI also needs the bare `m:ss` for the
/// now-playing row's two elapsed/duration labels. Both live here so they cannot
/// drift apart.
enum TimeFormat {
    /// `m:ss`, truncating sub-second precision.
    static func elapsed(_ ms: UInt64) -> String {
        let totalSeconds = ms / 1000
        return String(format: "%d:%02d", totalSeconds / 60, totalSeconds % 60)
    }

    /// `"position / duration"`, matching the core's `time_label` layout.
    static func pair(_ positionMs: UInt64, _ durationMs: UInt64) -> String {
        "\(elapsed(positionMs)) / \(elapsed(durationMs))"
    }
}
