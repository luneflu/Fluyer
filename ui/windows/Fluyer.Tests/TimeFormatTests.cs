using Fluyer.Core.Support;

namespace Fluyer.Tests;

public sealed class TimeFormatTests
{
    [Theory]
    [InlineData(0UL, "0:00")]
    [InlineData(999UL, "0:00")]
    [InlineData(1000UL, "0:01")]
    [InlineData(59_999UL, "0:59")]
    [InlineData(60_000UL, "1:00")]
    [InlineData(3_600_000UL, "60:00")]
    [InlineData(185_730UL, "3:05")]
    public void Elapsed_TruncatesSubSecond(ulong ms, string expected)
        => Assert.Equal(expected, TimeFormat.Elapsed(ms));

    [Fact]
    public void Pair_MatchesCoreTimeLabelLayout()
        => Assert.Equal("1:30 / 3:00", TimeFormat.Pair(90_000, 180_000));

    [Fact]
    public void Pair_ZeroZero()
        => Assert.Equal("0:00 / 0:00", TimeFormat.Pair(0, 0));
}
