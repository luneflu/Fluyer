using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Core.Support;

namespace Fluyer.Tests;

public sealed class PlaybackStateTests
{
    private static (PlaybackState State, FakeEngine Engine) Create()
    {
        var engine = new FakeEngine();
        var state = new PlaybackState { Engine = engine };
        return (state, engine);
    }

    [Fact]
    public void ReloadBar_AdoptsSnapshotWhole_IncludingVolume()
    {
        var (state, engine) = Create();
        engine.BarView = () => ViewModelDefaults.NoTrack with
        {
            Title = "Song", Volume = 0.4f, IsPlaying = true,
            PositionMs = 10_000, DurationMs = 100_000,
        };
        state.ReloadBar();
        Assert.Equal("Song", state.Bar.Title);
        Assert.Equal(0.4f, state.Bar.Volume);
        Assert.True(state.Bar.IsPlaying);
        Assert.Equal(10_000UL, state.Clock.PositionMs);
        state.Clock.Stop();
    }

    [Fact]
    public void PlayerBar_OverlaysClockPosition()
    {
        var (state, _) = Create();
        state.ApplyBar(ViewModelDefaults.NoTrack with { DurationMs = 100_000 });
        state.Clock.ApplyLocalPosition(25_000);
        var overlay = state.PlayerBar;
        Assert.Equal(25_000UL, overlay.PositionMs);
        Assert.Equal("0:25 / 1:40", overlay.TimeLabel);
        Assert.Equal("No Track", overlay.Title);
    }

    [Fact]
    public void TogglePlay_IsOptimistic()
    {
        var (state, engine) = Create();
        state.TogglePlay();
        Assert.Contains("toggle", engine.Calls);
        Assert.True(state.Bar.IsPlaying);
        state.TogglePlay();
        Assert.False(state.Bar.IsPlaying);
        state.Clock.Stop();
    }

    [Fact]
    public void Play_OnlyTogglesWhenPaused()
    {
        var (state, engine) = Create();
        state.Play(); // paused -> plays
        Assert.True(state.Bar.IsPlaying);
        Assert.Equal(1, engine.Calls.Count(c => c == "toggle"));
        state.Play(); // already playing -> no-op
        Assert.Equal(1, engine.Calls.Count(c => c == "toggle"));
        state.Clock.Stop();
    }

    [Fact]
    public void Pause_OnlyTogglesWhenPlaying()
    {
        var (state, engine) = Create();
        state.Pause(); // already paused -> no-op
        Assert.Empty(engine.Calls);
        state.Play(); // plays
        state.Pause(); // pauses
        Assert.False(state.Bar.IsPlaying);
        Assert.Equal(2, engine.Calls.Count(c => c == "toggle"));
        state.Clock.Stop();
    }

    [Fact]
    public void SetVolume_ClampsAndRemembersAudible()
    {
        var (state, engine) = Create();
        state.SetVolume(2.0f);
        Assert.Equal(1.0f, state.Bar.Volume);
        Assert.Equal(1.0f, engine.VolumeSets[^1]);

        state.SetVolume(0.0f); // mute
        state.ToggleMute(); // restore
        Assert.Equal(1.0f, state.Bar.Volume);

        state.SetVolume(0.002f); // barely audible — recorded as last audible
        state.ToggleMute(); // mute
        Assert.Equal(0.0f, state.Bar.Volume);
        state.ToggleMute(); // restore floors to the minimum restore volume
        Assert.Equal(0.05f, state.Bar.Volume);
    }

    [Fact]
    public void SeekFraction_InertWithoutDuration()
    {
        var (state, engine) = Create();
        state.SeekFraction(0.5f);
        Assert.DoesNotContain("seek", engine.Calls);
    }

    [Fact]
    public void SeekFraction_SeeksScaledTarget()
    {
        var (state, engine) = Create();
        state.ApplyBar(ViewModelDefaults.NoTrack with { DurationMs = 200_000 });
        state.SeekFraction(0.25f);
        Assert.Equal(new[] { 50_000UL }, engine.SeekTargets);
        state.Clock.Stop();
    }

    [Fact]
    public void ShowPlayView_DrivesLyricFollowing()
    {
        var (state, engine) = Create();
        engine.LyricIndex = _ => 3;
        state.ApplyBar(ViewModelDefaults.NoTrack with { DurationMs = 100_000 });
        state.ShowPlayView = true;
        Assert.True(state.Clock.FollowsLyrics);
        Assert.Equal(3, state.Clock.CurrentLyricIndex);
        state.ShowPlayView = false;
        Assert.False(state.Clock.FollowsLyrics);
        state.Clock.Stop();
    }

    [Fact]
    public void ApplyPlayView_AdoptsSnapshot()
    {
        var (state, _) = Create();
        var view = ViewModelDefaults.EmptyPlayView with
        {
            Lyrics = [new LyricLine(0, "hello")],
        };
        state.ApplyPlayView(view);
        Assert.Single(state.PlayView.Lyrics);
    }

    [Fact]
    public void NullEngine_IsInert()
    {
        var state = new PlaybackState { Engine = null };
        state.ReloadBar();
        state.ReloadPlayView();
        state.ApplyTrackChange();
        state.Seek(1000);
        state.Next();
        Assert.Equal(ViewModelDefaults.NoTrack, state.Bar);
    }
}
