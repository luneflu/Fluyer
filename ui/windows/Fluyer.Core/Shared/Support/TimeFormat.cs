namespace Fluyer.Core.Support;

/// <summary>
/// Port of <c>TimeFormat</c> (<c>ui/macos/Sources/Shared/Support/TimeFormat.swift</c>).
/// Mirrors the core's <c>view_models::format_time</c> exactly: <c>m:ss</c> with
/// truncated sub-second precision.
/// </summary>
public static class TimeFormat
{
    public static string Elapsed(ulong ms)
    {
        var totalSeconds = ms / 1000;
        return $"{totalSeconds / 60}:{totalSeconds % 60:D2}";
    }

    public static string Pair(ulong positionMs, ulong durationMs)
        => $"{Elapsed(positionMs)} / {Elapsed(durationMs)}";
}
