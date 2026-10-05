using Fluyer.Core.Native;
using Fluyer.Core.Support;

namespace Fluyer.Core.State;

/// <summary>
/// Now-playing state: metadata, transport flags, the position clock and every
/// command the UI issues to the player. Port of <c>PlaybackState</c>
/// (<c>ui/macos/Sources/State/PlaybackState.swift</c>).
/// </summary>
public sealed class PlaybackState : Support.ObservableObject
{
    private const float AudibleThreshold = 0.001f;
    private const float MinimumRestoreVolume = 0.05f;

    private PlayerBarViewModel _bar = ViewModelDefaults.NoTrack;
    private PlayViewModel _playView = ViewModelDefaults.EmptyPlayView;
    private bool _showPlayView;
    private float _lastAudibleVolume = 1.0f;

    public PlaybackClock Clock { get; } = new();

    public IFluyerEngine? Engine { get; set; }

    /// <summary>
    /// Metadata and transport flags. Position is deliberately *not* read from
    /// here — <see cref="Clock"/> owns it; <see cref="PlayerBar"/> overlays it.
    /// </summary>
    public PlayerBarViewModel Bar
    {
        get => _bar;
        private set
        {
            if (SetProperty(ref _bar, value))
            {
                OnPropertyChanged(nameof(PlayerBar));
            }
        }
    }

    public PlayViewModel PlayView
    {
        get => _playView;
        private set => SetProperty(ref _playView, value);
    }

    public bool ShowPlayView
    {
        get => _showPlayView;
        set
        {
            if (!SetProperty(ref _showPlayView, value))
            {
                return;
            }
            Clock.FollowsLyrics = value;
            if (value)
            {
                SyncLyricCursor();
            }
        }
    }

    public PlaybackState()
    {
        Clock.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName is nameof(PlaybackClock.PositionMs)
                or nameof(PlaybackClock.DurationMs)
                or nameof(PlaybackClock.ProgressPct)
                or nameof(PlaybackClock.TimeLabel))
            {
                OnPropertyChanged(nameof(PlayerBar));
            }
        };
    }

    /// <summary>Core player-bar snapshot with the locally-ticked position filled in.</summary>
    public PlayerBarViewModel PlayerBar => Bar with
    {
        PositionMs = Clock.PositionMs,
        DurationMs = Clock.DurationMs,
        ProgressPct = Clock.ProgressPct,
        TimeLabel = Clock.TimeLabel,
    };

    // MARK: - Core refreshes

    /// <summary>
    /// Adopt an authoritative player-bar snapshot from the core.
    /// The snapshot from <c>get_player_bar_view</c> is complete and
    /// authoritative — including the real volume — so take it whole.
    /// </summary>
    public void ApplyBar(PlayerBarViewModel snapshot)
    {
        Bar = snapshot;
        Clock.Adopt(snapshot);
        if (ShowPlayView)
        {
            ReloadPlayView();
        }
        SyncClockToTransport();
    }

    public void ReloadBar()
    {
        if (Engine is null)
        {
            return;
        }
        ApplyBar(Engine.GetPlayerBarView());
    }

    public void ReloadPlayView()
    {
        if (Engine is null)
        {
            return;
        }
        PlayView = Engine.GetPlayView();
    }

    public void ApplyPlayView(PlayViewModel viewModel) => PlayView = viewModel;

    public void ApplyTrackChange()
    {
        if (Engine is null)
        {
            return;
        }
        ReloadBar();
        ReloadPlayView();
        SyncLyricCursor();
    }

    // MARK: - Transport commands

    public void TogglePlay()
    {
        Engine?.TogglePlay();
        // Optimistic: the core confirms with its own player-sync event.
        Bar = Bar with { IsPlaying = !Bar.IsPlaying };
        SyncClockToTransport();
    }

    public void Play()
    {
        if (!Bar.IsPlaying)
        {
            TogglePlay();
        }
    }

    public void Pause()
    {
        if (Bar.IsPlaying)
        {
            TogglePlay();
        }
    }

    public void Next() => Engine?.Next();

    public void Previous() => Engine?.Previous();

    public void CycleRepeat() => Engine?.CycleRepeat();

    public void Shuffle() => Engine?.Shuffle();

    public void Seek(ulong positionMs)
    {
        if (Engine is null)
        {
            return;
        }
        Engine.Seek(positionMs);
        Clock.ApplyLocalPosition(positionMs);
        SyncLyricCursor();
    }

    /// <summary>Seek from a fraction of the track, as produced by dragging the progress bar.</summary>
    public void SeekFraction(float fraction)
    {
        var target = Clock.TargetPositionMs(fraction);
        if (target.HasValue)
        {
            Seek(target.Value);
        }
    }

    public void SetVolume(float volume)
    {
        var clamped = volume.Clamped(0.0f, 1.0f);
        if (clamped > AudibleThreshold)
        {
            _lastAudibleVolume = clamped;
        }
        Bar = Bar with { Volume = clamped };
        Engine?.SetVolume(clamped);
    }

    public void ToggleMute()
    {
        if (Bar.Volume > AudibleThreshold)
        {
            SetVolume(0.0f);
        }
        else
        {
            SetVolume(Math.Max(_lastAudibleVolume, MinimumRestoreVolume));
        }
    }

    // MARK: - Clock

    private void SyncClockToTransport()
    {
        if (Bar.IsPlaying)
        {
            Clock.Start(Engine);
        }
        else
        {
            Clock.Stop();
        }
    }

    private void SyncLyricCursor()
    {
        if (!ShowPlayView || Engine is null)
        {
            return;
        }
        Clock.CurrentLyricIndex = Engine.GetActiveLyricIndex(Clock.PositionMs);
    }
}
