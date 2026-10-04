using Fluyer.Core.Native;
using Fluyer.Core.Support;

namespace Fluyer.Core.State;

/// <summary>
/// Locally-ticked playback position and the active lyric cursor. Port of
/// <c>PlaybackClock</c> (<c>ui/macos/Sources/State/PlaybackClock.swift</c>).
///
/// The core only emits player sync on discrete transport actions, never on a
/// timer, so continuous progress is sampled here (~4x/s). Kept separate from
/// <see cref="PlaybackState.Bar"/> so a tick invalidates only position
/// readers, not everything bound to the player bar.
/// </summary>
public sealed class PlaybackClock : Support.ObservableObject
{
    private static readonly TimeSpan TickInterval = TimeSpan.FromMilliseconds(250);

    private ulong _positionMs;
    private ulong _durationMs;
    private float _progressPct;
    private string _timeLabel = TimeFormat.Pair(0, 0);
    private int _currentLyricIndex = -1;
    private bool _followsLyrics;

    private IFluyerEngine? _engine;
    private System.Threading.Timer? _timer;
    private SynchronizationContext? _context;

    public ulong PositionMs
    {
        get => _positionMs;
        private set => SetProperty(ref _positionMs, value);
    }

    public ulong DurationMs
    {
        get => _durationMs;
        private set => SetProperty(ref _durationMs, value);
    }

    public float ProgressPct
    {
        get => _progressPct;
        private set => SetProperty(ref _progressPct, value);
    }

    public string TimeLabel
    {
        get => _timeLabel;
        private set => SetProperty(ref _timeLabel, value);
    }

    public int CurrentLyricIndex
    {
        get => _currentLyricIndex;
        set => SetProperty(ref _currentLyricIndex, value);
    }

    public bool FollowsLyrics
    {
        get => _followsLyrics;
        set => SetProperty(ref _followsLyrics, value);
    }

    public void Start(IFluyerEngine? engine)
    {
        _engine = engine;
        // RunLoop.common equivalent: a threadpool timer keeps firing while a
        // menu or window drag holds the UI thread, so the progress bar never
        // freezes mid-track. Ticks hop back via _context.
        if (_timer is not null || engine is null)
        {
            return;
        }
        _context = SynchronizationContext.Current;
        _timer = new System.Threading.Timer(_ => OnTick(), null, TickInterval, TickInterval);
    }

    public void Stop()
    {
        _timer?.Dispose();
        _timer = null;
    }

    /// <summary>Adopt the position fields of an authoritative core snapshot verbatim.</summary>
    public void Adopt(PlayerBarViewModel snapshot)
    {
        PositionMs = snapshot.PositionMs;
        DurationMs = snapshot.DurationMs;
        ProgressPct = snapshot.ProgressPct;
        TimeLabel = snapshot.TimeLabel;
    }

    /// <summary>
    /// Recompute derived position fields after a local seek, before the core
    /// echoes back. Mirrors the core's own clamping in <c>lib.rs</c>.
    /// </summary>
    public void ApplyLocalPosition(ulong positionMs)
    {
        PositionMs = positionMs;
        if (DurationMs == 0)
        {
            return;
        }
        ProgressPct = (float)positionMs / DurationMs;
        TimeLabel = TimeFormat.Pair(positionMs, DurationMs);
    }

    /// <summary>
    /// Position targeted by a drag on the progress bar, or <c>null</c> when
    /// there is no duration to seek within.
    /// </summary>
    public ulong? TargetPositionMs(float fraction)
    {
        if (DurationMs == 0)
        {
            return null;
        }
        var clamped = fraction.Clamped(0.0f, 1.0f);
        return (ulong)(DurationMs * clamped);
    }

    private void OnTick()
    {
        if (_context is null)
        {
            Tick();
        }
        else
        {
            _context.Post(_ => Tick(), null);
        }
    }

    private void Tick()
    {
        if (_engine is null)
        {
            return;
        }
        var position = _engine.GetPosition();
        PositionMs = position;
        if (DurationMs == 0)
        {
            return;
        }
        ProgressPct = (float)position / DurationMs;
        TimeLabel = TimeFormat.Pair(position, DurationMs);
        if (FollowsLyrics)
        {
            CurrentLyricIndex = _engine.GetActiveLyricIndex(position);
        }
    }
}
