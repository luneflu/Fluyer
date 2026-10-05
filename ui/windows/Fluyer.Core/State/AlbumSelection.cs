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

    private string _query = string.Empty;

    /// <summary>Search text; matches title, artist or album (case-insensitive).</summary>
    public string Query
    {
        get => _query;
        set
        {
            if (SetProperty(ref _query, value ?? string.Empty))
            {
                OnPropertyChanged(nameof(DisplayedTracks));
            }
        }
    }

    private IReadOnlyList<TrackItemViewModel> Source =>
        (IReadOnlyList<TrackItemViewModel>?)Detail?.Tracks ?? _library.Tracks;

    /// <summary>Rows the grid renders: the album's tracks, else the whole library, narrowed by <see cref="Query"/>.</summary>
    // ponytail: filtered result is a snapshot, so the now-playing marker under an
    // active query refreshes on the next keystroke; make it live if that bothers.
    public IReadOnlyList<TrackItemViewModel> DisplayedTracks
    {
        get
        {
            var q = Query.Trim();
            return q.Length == 0 ? Source : Source.Where(t => Matches(t, q)).ToList();
        }
    }

    internal static bool Matches(TrackItemViewModel t, string q)
        => t.Title.Contains(q, StringComparison.OrdinalIgnoreCase)
        || t.Artist.Contains(q, StringComparison.OrdinalIgnoreCase)
        || t.Album.Contains(q, StringComparison.OrdinalIgnoreCase);

    /// <summary>Play a displayed track; resolves its row in the unfiltered list so the queue stays whole.</summary>
    public void PlayTrack(TrackItemViewModel track)
    {
        var source = Source;
        for (var row = 0; row < source.Count; row++)
        {
            if (source[row].Path == track.Path)
            {
                PlayTrackAtRow(row);
                return;
            }
        }
    }

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
        if (Query.Length > 0)
        {
            // Library contents changed under an active filter snapshot.
            OnPropertyChanged(nameof(DisplayedTracks));
        }
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
