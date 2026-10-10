using System.Globalization;
using Fluyer.Core.Native;

namespace Fluyer.Core.State;

/// <summary>What the library pane lists (legacy <c>MusicListType</c>).</summary>
// ponytail: no Playlist / Folder yet; the core has no playlist or folder API.
public enum LibraryMode
{
    /// <summary>Album carousel over the track grid.</summary>
    Tracks,
    /// <summary>Full-height album grid.</summary>
    Albums,
}

/// <summary>Track sort keys. <see cref="Album"/> is the core's library order (album, track number, file name).</summary>
public enum TrackSort { Album, Title, Artist, Duration }

public enum AlbumSort { Name, Artist, Year, TrackCount }

/// <summary>
/// Library mode, search, sort, and which album the track grid is showing, plus
/// album-scoped playback commands. Port of <c>LibraryFilterState</c>
/// (<c>ui/macos/Sources/Screens/Home/LibraryFilterState.swift</c>).
/// <c>Index</c> is the single source of truth; <c>Detail</c> is derived from it.
/// </summary>
public sealed class LibraryFilterState : Support.ObservableObject
{
    private readonly LibraryState _library;

    private int? _index;
    private AlbumDetailViewModel? _detail;
    private LibraryMode _mode = LibraryMode.Tracks;
    private string _query = string.Empty;
    private TrackSort _trackSort = TrackSort.Album;
    private AlbumSort _albumSort = AlbumSort.Name;
    private bool _sortAscending = true;

    public IFluyerEngine? Engine { get; set; }

    public LibraryFilterState(LibraryState library)
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

    public LibraryMode Mode
    {
        get => _mode;
        set => SetProperty(ref _mode, value);
    }

    /// <summary>Search text; matches title, artist or album (case- and diacritic-insensitive).</summary>
    public string Query
    {
        get => _query;
        set
        {
            if (SetProperty(ref _query, value ?? string.Empty))
            {
                OnPropertyChanged(nameof(DisplayedTracks));
                OnPropertyChanged(nameof(DisplayedAlbums));
            }
        }
    }

    public TrackSort TrackSort
    {
        get => _trackSort;
        set
        {
            if (SetProperty(ref _trackSort, value))
            {
                OnPropertyChanged(nameof(DisplayedTracks));
            }
        }
    }

    public AlbumSort AlbumSort
    {
        get => _albumSort;
        set
        {
            if (SetProperty(ref _albumSort, value))
            {
                OnPropertyChanged(nameof(DisplayedAlbums));
            }
        }
    }

    /// <summary>Shared by both modes, like legacy <c>filterBarStore.sortAsc</c>.</summary>
    // ponytail: sort is session-only; persist in SettingsState if users ask.
    public bool SortAscending
    {
        get => _sortAscending;
        set
        {
            if (SetProperty(ref _sortAscending, value))
            {
                OnPropertyChanged(nameof(DisplayedTracks));
                OnPropertyChanged(nameof(DisplayedAlbums));
            }
        }
    }

    /// <summary>Albums whose name or artist matches <see cref="Query"/>, sorted.</summary>
    // ponytail: legacy kept every album holding a matching track; name/artist only
    // is enough until someone misses it, then match via the library tracks.
    public IReadOnlyList<AlbumCardViewModel> DisplayedAlbums
    {
        get
        {
            var q = Query.Trim();
            IEnumerable<AlbumCardViewModel> albums = _library.Albums;
            if (q.Length > 0)
            {
                albums = albums.Where(a => Matches([a.Name, a.Artist], q));
            }
            var sorted = albums.Order(Comparer<AlbumCardViewModel>.Create((a, b) => Compare(a, b, AlbumSort))).ToList();
            if (!SortAscending)
            {
                sorted.Reverse();
            }
            return sorted;
        }
    }

    /// <summary>Unfiltered rows: the album's tracks when one is selected, else the library.</summary>
    private IReadOnlyList<TrackItemViewModel> Source =>
        (IReadOnlyList<TrackItemViewModel>?)Detail?.Tracks ?? _library.Tracks;

    /// <summary>The rows the grid renders: <see cref="Source"/> sorted, then narrowed by <see cref="Query"/>.</summary>
    // ponytail: sorted/filtered results are re-sorted on every read and handed out as a
    // snapshot, so the now-playing marker there refreshes on track change (RefreshDisplayed).
    // Cache on source/sort changes if big libraries stutter.
    public IReadOnlyList<TrackItemViewModel> DisplayedTracks
    {
        get
        {
            var q = Query.Trim();
            // Default order, no search: hand out the live collection so the
            // now-playing marker (LibraryState.ReloadActiveFlags) updates in place.
            if (q.Length == 0 && TrackSort == TrackSort.Album && SortAscending)
            {
                return Source;
            }
            var tracks = Sorted(Source);
            return q.Length == 0 ? tracks : tracks.Where(t => Matches(t, q)).ToList();
        }
    }

    private List<TrackItemViewModel> Sorted(IReadOnlyList<TrackItemViewModel> tracks)
    {
        var ordered = TrackSort == TrackSort.Album
            ? tracks.ToList()
            : tracks.Order(Comparer<TrackItemViewModel>.Create((a, b) => Compare(a, b, TrackSort))).ToList();
        if (!SortAscending)
        {
            ordered.Reverse();
        }
        return ordered;
    }

    // Index breaks ties so equal keys keep library order.
    internal static int Compare(TrackItemViewModel a, TrackItemViewModel b, TrackSort sort) => sort switch
    {
        TrackSort.Title => Natural(a.Title, b.Title, a.Index, b.Index),
        TrackSort.Artist => Natural(a.Artist, b.Artist, a.Index, b.Index),
        TrackSort.Duration => (a.DurationMs, a.Index).CompareTo((b.DurationMs, b.Index)),
        _ => a.Index.CompareTo(b.Index),
    };

    internal static int Compare(AlbumCardViewModel a, AlbumCardViewModel b, AlbumSort sort) => sort switch
    {
        AlbumSort.Artist => Natural(a.Artist, b.Artist, a.Index, b.Index),
        AlbumSort.Year => Natural(a.Year, b.Year, a.Index, b.Index),
        AlbumSort.TrackCount => (a.TrackCount, a.Index).CompareTo((b.TrackCount, b.Index)),
        _ => Natural(a.Name, b.Name, a.Index, b.Index),
    };

    /// <summary>Explorer/Finder order: case-insensitive, numbers by value ("2" before "10").</summary>
    private static int Natural(string a, string b, ulong tieA, ulong tieB)
    {
        var c = NaturalCompare(a, b);
        return c != 0 ? c : tieA.CompareTo(tieB);
    }

    // .NET 8 has no CompareOptions.NumericOrdering (added in .NET 10): compare digit
    // runs by value, everything else culture-aware and case-insensitive.
    internal static int NaturalCompare(string a, string b)
    {
        int i = 0, j = 0;
        while (i < a.Length && j < b.Length)
        {
            if (char.IsAsciiDigit(a[i]) && char.IsAsciiDigit(b[j]))
            {
                int si = i, sj = j;
                while (i < a.Length && char.IsAsciiDigit(a[i])) i++;
                while (j < b.Length && char.IsAsciiDigit(b[j])) j++;
                var da = a[si..i].TrimStart('0');
                var db = b[sj..j].TrimStart('0');
                var c = da.Length != db.Length ? da.Length.CompareTo(db.Length) : string.CompareOrdinal(da, db);
                if (c != 0) return c;
            }
            else
            {
                int si = i, sj = j;
                while (i < a.Length && !char.IsAsciiDigit(a[i])) i++;
                while (j < b.Length && !char.IsAsciiDigit(b[j])) j++;
                var c = string.Compare(a[si..i], b[sj..j], CultureInfo.CurrentCulture, CompareOptions.IgnoreCase);
                if (c != 0) return c;
            }
        }
        return (a.Length - i).CompareTo(b.Length - j);
    }

    internal static bool Matches(TrackItemViewModel t, string q) => Matches([t.Title, t.Artist, t.Album], q);

    private static bool Matches(string[] fields, string q)
        => fields.Any(f => CultureInfo.CurrentCulture.CompareInfo.IndexOf(f, q,
            CompareOptions.IgnoreCase | CompareOptions.IgnoreNonSpace) >= 0);

    /// <summary>
    /// Play a displayed track. The queue is the sorted, unfiltered list, so it
    /// follows the grid's order and stays whole under an active search.
    /// </summary>
    public void PlayTrack(TrackItemViewModel track) => Play(Sorted(Source), track.Path);

    /// <summary>Hands the core library indices, resolved by path: album rows carry album-local indices.</summary>
    private void Play(IReadOnlyList<TrackItemViewModel> queue, string? startPath)
    {
        var byPath = new Dictionary<string, ulong>();
        foreach (var t in _library.Tracks)
        {
            byPath.TryAdd(t.Path, t.Index);
        }
        var indices = new List<ulong>();
        var start = 0;
        foreach (var t in queue)
        {
            if (byPath.TryGetValue(t.Path, out var i))
            {
                if (t.Path == startPath)
                {
                    start = indices.Count;
                }
                indices.Add(i);
            }
        }
        if (indices.Count > 0)
        {
            Engine?.PlayLibraryTracks(indices, (ulong)start);
        }
    }

    /// <summary>Open an album's tracks; from the album grid this returns to the track view.</summary>
    public void Select(int index)
    {
        Index = index;
        Mode = LibraryMode.Tracks;
        Detail = Engine?.GetAlbumDetail((ulong)index);
    }

    public void Clear()
    {
        Index = null;
        Detail = null;
    }

    /// <summary>Re-announce the derived lists (library reloaded, or the playing track changed).</summary>
    public void RefreshDisplayed()
    {
        OnPropertyChanged(nameof(DisplayedTracks));
        OnPropertyChanged(nameof(DisplayedAlbums));
    }

    /// <summary>Re-read the selected album after a library rescan.</summary>
    public void Reload()
    {
        RefreshDisplayed();
        if (!Index.HasValue)
        {
            Detail = null;
            return;
        }
        Detail = Engine?.GetAlbumDetail((ulong)Index.Value);
    }

    // MARK: - Playback

    /// <summary>Header "Play": the open album in the grid's order.</summary>
    public void PlaySelected()
    {
        if (Detail is { } detail)
        {
            Play(Sorted(detail.Tracks), null);
        }
    }

    public void QueueSelected()
    {
        if (Index is { } index)
        {
            QueueAlbum(index);
        }
    }

    public void ShuffleSelected()
    {
        if (Index is { } index)
        {
            ShuffleAlbum(index);
        }
    }

    public void PlayAlbum(int index) => Engine?.PlayAlbum((ulong)index);
    public void QueueAlbum(int index) => Engine?.QueueAlbum((ulong)index);
    public void ShuffleAlbum(int index) => Engine?.ShuffleAlbum((ulong)index);
}
