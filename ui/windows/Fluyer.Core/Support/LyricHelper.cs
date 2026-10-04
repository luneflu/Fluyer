using Fluyer.Core.Native;

namespace Fluyer.Core.Support;

/// <summary>
/// Client-side mirror of <c>view_models::find_active_lyric_index</c>: last line
/// with <c>timestamp_ms &lt;= position</c>, <c>0</c> when before the first line,
/// <c>-1</c> when there are no lines.
/// </summary>
public static class LyricHelper
{
    public static int FindActiveIndex(IReadOnlyList<LyricLine> lyrics, ulong positionMs)
    {
        if (lyrics.Count == 0)
        {
            return -1;
        }
        if (positionMs < lyrics[0].TimestampMs)
        {
            return 0;
        }
        // Partition point: first line past positionMs (binary search).
        var lo = 0;
        var hi = lyrics.Count;
        while (lo < hi)
        {
            var mid = lo + ((hi - lo) >> 1);
            if (lyrics[mid].TimestampMs <= positionMs)
            {
                lo = mid + 1;
            }
            else
            {
                hi = mid;
            }
        }
        return lo - 1;
    }
}
