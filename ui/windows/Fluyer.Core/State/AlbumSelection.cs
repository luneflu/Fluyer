using Fluyer.Core.Native;

namespace Fluyer.Core.State;

/// <summary>
/// Which album the track grid is showing, plus album-scoped playback commands.
/// Port of <c>AlbumSelection</c> (<c>ui/macos/Sources/State/AlbumSelection.swift</c>).
/// <c>Index</c> is the single source of truth; <c>Detail</c> is derived from it.
/// </summary>
public sealed class AlbumSelection : Support.ObservableObject
{
    private readonly LibraryState _library;

    private int? _index;
    private AlbumDetailViewModel? _detail;

    public IFluyerEngine? Engine { get; set; }

    public AlbumSelection(LibraryState library)
    {
        _library = library;
    }

    public int? Index
    {
        get => _index;
        private set
        {
            if (SetProperty(ref _index, value))
            {
                OnPropertyChanged(nameof(IsActive));
                OnPropertyChanged(nameof(DisplayedTracks));
            }
        }
    }

    public AlbumDetailViewModel? Detail
    {
        get => _detail;
        private set
        {
            if (SetProperty(ref _detail, value))
            {
                OnPropertyChanged(nameof(DisplayedTracks));
            }
        }
    }

    public bool IsActive => Index.HasValue;

    /// <summary>Rows the grid renders: the album's tracks, else the whole library.</summary>
    public IReadOnlyList<TrackItemViewModel> DisplayedTracks =>
        (IReadOnlyList<TrackItemViewModel>?)Detail?.Tracks ?? _library.Tracks;

    public void Select(int index)
    {
        Index = index;
        Detail = Engine?.GetAlbumDetail((ulong)index);
    }

    public void Clear()
    {
        Index = null;
        Detail = null;
    }

    /// <summary>Re-read the selected album after a library rescan.</summary>
    public void Reload()
    {
        if (!Index.HasValue)
        {
            Detail = null;
            return;
        }
        Detail = Engine?.GetAlbumDetail((ulong)Index.Value);
    }

    /// <summary>Play a row of <see cref="DisplayedTracks"/>, resolving against the selected album.</summary>
    public void PlayTrackAtRow(int row)
    {
        if (row < 0)
        {
            return;
        }
        if (Index.HasValue)
        {
            Engine?.PlayAlbumTrack((ulong)Index.Value, (ulong)row);
        }
        else
        {
            Engine?.PlayAllFromLibrary((ulong)row);
        }
    }

    public void PlaySelected()
    {
        if (Index.HasValue)
        {
            Engine?.PlayAlbum((ulong)Index.Value);
        }
    }

    public void QueueSelected()
    {
        if (Index.HasValue)
        {
            Engine?.QueueAlbum((ulong)Index.Value);
        }
    }

    public void ShuffleSelected()
    {
        if (Index.HasValue)
        {
            Engine?.ShuffleAlbum((ulong)Index.Value);
        }
    }
}
