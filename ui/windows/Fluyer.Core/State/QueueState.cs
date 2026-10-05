using Fluyer.Core.Native;

namespace Fluyer.Core.State;

/// <summary>
/// Play queue snapshot plus queue commands. Row <c>Index</c> is the queue
/// position (not a library index). Only re-read while <see cref="IsOpen"/>,
/// so player-sync bursts don't serialize the whole queue for a hidden panel.
/// </summary>
public sealed class QueueState : Support.ObservableObject
{
    private IReadOnlyList<TrackItemViewModel> _tracks = Array.Empty<TrackItemViewModel>();
    private bool _isOpen;

    public IFluyerEngine? Engine { get; set; }

    public IReadOnlyList<TrackItemViewModel> Tracks
    {
        get => _tracks;
        private set => SetProperty(ref _tracks, value);
    }

    public bool IsOpen
    {
        get => _isOpen;
        set
        {
            if (SetProperty(ref _isOpen, value) && value)
            {
                Reload();
            }
        }
    }

    public void Reload()
    {
        if (!IsOpen || Engine is null)
        {
            return;
        }
        Tracks = Engine.GetQueue();
    }

    public void Goto(int index)
    {
        if (index >= 0 && index < Tracks.Count)
        {
            Engine?.QueueGoto((ulong)index);
        }
    }

    public void Remove(int index)
    {
        if (index < 0 || index >= Tracks.Count)
        {
            return;
        }
        Engine?.QueueRemove((ulong)index);
        Reload();
    }

    /// <summary>Move by <paramref name="delta"/> rows; no-op past either end.</summary>
    public void Move(int index, int delta)
    {
        var to = index + delta;
        if (index < 0 || index >= Tracks.Count || to < 0 || to >= Tracks.Count)
        {
            return;
        }
        Engine?.QueueMove((ulong)index, (ulong)to);
        Reload();
    }
}
