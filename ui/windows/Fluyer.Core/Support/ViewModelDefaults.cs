using Fluyer.Core.Native;

namespace Fluyer.Core.Support;

/// <summary>
/// Port of <c>ViewModelDefaults</c> (<c>ui/macos/Sources/Support/ViewModelDefaults.swift</c>).
/// Placeholders for the window that renders before the core reports anything;
/// mirror the Rust constructors exactly (no stray "Unknown Title").
/// </summary>
public static class ViewModelDefaults
{
    public static PlayerBarViewModel NoTrack { get; } = new(
        TrackIndex: -1,
        Title: "No Track",
        Artist: string.Empty,
        Album: string.Empty,
        PositionMs: 0,
        DurationMs: 0,
        ProgressPct: 0.0f,
        TimeLabel: "0:00 / 0:00",
        IsPlaying: false,
        RepeatMode: RepeatMode.None,
        IsShuffled: false,
        Volume: 1.0f);

    public static PlayViewModel EmptyPlayView { get; } = new(
        Track: null,
        Lyrics: [],
        CurrentLyricIndex: -1,
        Palette: [new ColorRgb(28, 28, 36)]);

    public static ScanStatusViewModel Idle { get; } = new(
        IsScanning: false,
        Current: 0,
        Total: 0,
        ProgressPct: 0.0f,
        StatusLabel: "Ready");
}
