namespace Fluyer.Core.Support;

/// <summary>
/// Port of <c>FloatingPoint.clamped</c>
/// (<c>ui/macos/Sources/Support/FloatingPoint+Clamped.swift</c>).
/// NaN collapses to the lower bound instead of propagating: every caller feeds
/// the result into an integer conversion (or a layout width) that cannot
/// represent it — the zero-width drag case is a real way to get there.
/// </summary>
public static class FloatClamp
{
    public static float Clamped(this float value, float min, float max)
    {
        if (float.IsNaN(value))
        {
            return min;
        }
        return Math.Min(Math.Max(value, min), max);
    }

    public static double Clamped(this double value, double min, double max)
    {
        if (double.IsNaN(value))
        {
            return min;
        }
        return Math.Min(Math.Max(value, min), max);
    }
}
