using Fluyer.Core.Native;

namespace Fluyer.Tests;

public sealed class LyricHelperTests
{
    private static List<LyricLine> Lyrics(params (ulong Ms, string Text)[] lines)
        => lines.Select(l => new LyricLine(l.Ms, l.Text)).ToList();

    [Fact]
    public void Empty_ReturnsMinusOne()
        => Assert.Equal(-1, Fluyer.Core.Support.LyricHelper.FindActiveIndex([], 60_000));

    [Fact]
    public void BeforeFirst_ReturnsZero()
        => Assert.Equal(0, Fluyer.Core.Support.LyricHelper.FindActiveIndex(
            Lyrics((10_000, "a"), (20_000, "b")), 5_000));

    [Theory]
    [InlineData(0UL, 0)]
    [InlineData(10_000UL, 0)]
    [InlineData(15_000UL, 0)]
    [InlineData(20_000UL, 1)]
    [InlineData(99_000UL, 1)]
    public void BinarySearch_LastLineAtOrBeforePosition(ulong position, int expected)
        => Assert.Equal(expected, Fluyer.Core.Support.LyricHelper.FindActiveIndex(
            Lyrics((10_000, "a"), (20_000, "b")), position));
}
