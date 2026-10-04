using Fluyer.Core.Native;
using Fluyer.Core.Support;

namespace Fluyer.Tests;

public sealed class ViewModelDefaultsTests
{
    [Fact]
    public void NoTrack_MirrorsRustDefault()
    {
        var bar = ViewModelDefaults.NoTrack;
        Assert.Equal(-1, bar.TrackIndex);
        Assert.Equal("No Track", bar.Title);
        Assert.Equal(string.Empty, bar.Artist);
        Assert.Equal(string.Empty, bar.Album);
        Assert.Equal(0UL, bar.PositionMs);
        Assert.Equal(0UL, bar.DurationMs);
        Assert.Equal(0.0f, bar.ProgressPct);
        Assert.Equal("0:00 / 0:00", bar.TimeLabel);
        Assert.False(bar.IsPlaying);
        Assert.Equal(RepeatMode.None, bar.RepeatMode);
        Assert.False(bar.IsShuffled);
        Assert.Equal(1.0f, bar.Volume);
    }

    [Fact]
    public void EmptyPlayView_MirrorsRustEmpty()
    {
        var play = ViewModelDefaults.EmptyPlayView;
        Assert.Null(play.Track);
        Assert.Empty(play.Lyrics);
        Assert.Equal(-1, play.CurrentLyricIndex);
        Assert.Single(play.Palette);
        Assert.Equal(new ColorRgb(28, 28, 36), play.Palette[0]);
    }

    [Fact]
    public void Idle_MirrorsRustIdle()
    {
        var status = ViewModelDefaults.Idle;
        Assert.False(status.IsScanning);
        Assert.Equal(0UL, status.Current);
        Assert.Equal(0UL, status.Total);
        Assert.Equal(0.0f, status.ProgressPct);
        Assert.Equal("Ready", status.StatusLabel);
    }
}
