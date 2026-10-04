using Fluyer.Core.Native;
using Fluyer.Core.Support;

namespace Fluyer.Tests;

public sealed class PlaybackIconsTests
{
    [Theory]
    [InlineData(RepeatMode.One, "\uE8ED")]
    [InlineData(RepeatMode.All, "\uE8EE")]
    [InlineData(RepeatMode.None, "\uE8EE")]
    public void Repeat_Table(RepeatMode mode, string expected)
        => Assert.Equal(expected, PlaybackIcons.RepeatIcon(mode));

    [Theory]
    [InlineData(0.0f, "\uE74F")]
    [InlineData(0.001f, "\uE74F")]
    [InlineData(0.01f, "\uE993")]
    [InlineData(0.329f, "\uE993")]
    [InlineData(0.33f, "\uE994")]
    [InlineData(0.659f, "\uE994")]
    [InlineData(0.66f, "\uE995")]
    [InlineData(1.0f, "\uE995")]
    public void Volume_Bands(float level, string expected)
        => Assert.Equal(expected, PlaybackIcons.Volume(level));
}
