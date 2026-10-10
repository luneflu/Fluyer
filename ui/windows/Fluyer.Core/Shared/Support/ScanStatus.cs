using Fluyer.Core.Native;

namespace Fluyer.Core.Support;

/// <summary>
/// Client-side mirror of <c>ScanStatusViewModel::{idle,scanning,completed}</c>
/// (<c>crates/fluyer_core/src/view_models/scan_status.rs</c>). The core has no
/// scan-status query — progress arrives via <c>on_scan_progress</c> — so the UI
/// builds these snapshots itself from the event payload.
/// </summary>
public static class ScanStatus
{
    public static ScanStatusViewModel Idle() => ViewModelDefaults.Idle;

    public static ScanStatusViewModel Scanning(ulong current, ulong total)
    {
        var progress = total > 0 ? Math.Min(Math.Max((float)current / total, 0.0f), 1.0f) : 0.0f;
        var label = total > 0 ? $"Scanning: {current} / {total} files" : "Scanning library...";
        return new ScanStatusViewModel(
            IsScanning: true,
            Current: current,
            Total: total,
            ProgressPct: progress,
            StatusLabel: label);
    }

    public static ScanStatusViewModel Completed(ulong total) => new(
        IsScanning: false,
        Current: total,
        Total: total,
        ProgressPct: 1.0f,
        StatusLabel: $"Scanned {total} files");
}
