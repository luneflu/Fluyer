using Fluyer.Core.State;
using Fluyer.Core.Support;

namespace Fluyer.Tests;

public sealed class PlaybackClockTests
{
    [Fact]
    public void Adopt_IsVerbatim()
    {
        var clock = new PlaybackClock();
        var snapshot = ViewModelDefaults.NoTrack with
        {
            PositionMs = 42_000,
            DurationMs = 180_000,
            ProgressPct = 0.25f,
            TimeLabel = "0:42 / 3:00",
        };
        clock.Adopt(snapshot);
        Assert.Equal(42_000UL, clock.PositionMs);
        Assert.Equal(180_000UL, clock.DurationMs);
        Assert.Equal(0.25f, clock.ProgressPct);
        Assert.Equal("0:42 / 3:00", clock.TimeLabel);
    }

    [Fact]
    public void ApplyLocalPosition_RecomputesDerivedFields()
    {
        var clock = new PlaybackClock();
        clock.Adopt(ViewModelDefaults.NoTrack with { DurationMs = 200_000 });
        clock.ApplyLocalPosition(50_000);
        Assert.Equal(50_000UL, clock.PositionMs);
        Assert.Equal(0.25f, clock.ProgressPct);
        Assert.Equal("0:50 / 3:20", clock.TimeLabel);
    }

    [Fact]
    public void ApplyLocalPosition_ZeroDuration_KeepsPositionOnly()
    {
        var clock = new PlaybackClock();
        clock.ApplyLocalPosition(50_000);
        Assert.Equal(50_000UL, clock.PositionMs);
        Assert.Equal(0.0f, clock.ProgressPct);
    }

    [Fact]
    public void TargetPositionMs_ClampsAndRejectsNaN()
    {
        var clock = new PlaybackClock();
        clock.Adopt(ViewModelDefaults.NoTrack with { DurationMs = 100_000 });
        Assert.Equal(50_000UL, clock.TargetPositionMs(0.5f));
        Assert.Equal(0UL, clock.TargetPositionMs(-1.0f));
        Assert.Equal(100_000UL, clock.TargetPositionMs(2.0f));
        // Zero-width drag produces NaN — must collapse to 0, never trap.
        Assert.Equal(0UL, clock.TargetPositionMs(float.NaN));
        Assert.Equal(100_000UL, clock.TargetPositionMs(float.PositiveInfinity));
    }

    [Fact]
    public void TargetPositionMs_NilWithoutDuration()
    {
        var clock = new PlaybackClock();
        Assert.Null(clock.TargetPositionMs(0.5f));
    }
}
