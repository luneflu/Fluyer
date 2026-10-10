using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Core.Support;

namespace Fluyer.Tests;

public sealed class LibraryFilterStateTests
{
    private static (LibraryFilterState Sel, LibraryState Lib, FakeEngine Engine) Create()
    {
        var engine = new FakeEngine();
        engine.TrackList.AddRange([FakeEngine.Track(0), FakeEngine.Track(1)]);
        var library = new LibraryState { Engine = engine };
        library.Reload();
        var sel = new LibraryFilterState(library) { Engine = engine };
        return (sel, library, engine);
    }

    [Fact]
    public void Inactive_ShowsWholeLibrary()
    {
        var (sel, library, _) = Create();
        Assert.False(sel.IsActive);
        Assert.Equal(library.Tracks.Count, sel.DisplayedTracks.Count);
    }

    [Fact]
    public void Select_ShowsAlbumTracks()
    {
        var (sel, _, engine) = Create();
        var detail = new AlbumDetailViewModel(
            Header: FakeEngine.Card(0), DurationMs: 360_000,
            TotalDurationFormatted: "6:00", Subtitle: "Album 0",
            Tracks: [FakeEngine.Track(0), FakeEngine.Track(1)]);
        engine.AlbumDetail = _ => detail;
        sel.Select(0);
        Assert.True(sel.IsActive);
        Assert.Equal(2, sel.DisplayedTracks.Count);
        Assert.Equal("Album 0", sel.Detail!.Subtitle);
    }

    [Fact]
    public void Select_FailedDetail_FallsBackToLibrary()
    {
        var (sel, library, _) = Create();
        sel.Select(0); // AlbumDetail func returns null
        Assert.True(sel.IsActive);
        Assert.Null(sel.Detail);
        Assert.Equal(library.Tracks.Count, sel.DisplayedTracks.Count);
    }

    [Fact]
    public void Clear_Resets()
    {
        var (sel, _, engine) = Create();
        engine.AlbumDetail = _ => new AlbumDetailViewModel(
            FakeEngine.Card(0), 0, "0:00", "s", []);
        sel.Select(0);
        sel.Clear();
        Assert.False(sel.IsActive);
        Assert.Null(sel.Detail);
    }

    private static LibraryFilterState WithTracks(params TrackItemViewModel[] tracks)
    {
        var engine = new FakeEngine();
        engine.TrackList.AddRange(tracks);
        var library = new LibraryState { Engine = engine };
        library.Reload();
        return new LibraryFilterState(library) { Engine = engine };
    }

    [Fact]
    public void TrackSort_OrdersNaturallyAndReverses()
    {
        var sel = WithTracks(
            FakeEngine.Track(0) with { Title = "Track 10", DurationMs = 300 },
            FakeEngine.Track(1) with { Title = "track 2", DurationMs = 1000 },
            FakeEngine.Track(2) with { Title = "Alpha", DurationMs = 100 });

        Assert.Equal([0UL, 1, 2], sel.DisplayedTracks.Select(t => t.Index)); // library order
        sel.TrackSort = TrackSort.Title;
        Assert.Equal([2UL, 1, 0], sel.DisplayedTracks.Select(t => t.Index)); // "2" before "10"
        sel.SortAscending = false;
        Assert.Equal([0UL, 1, 2], sel.DisplayedTracks.Select(t => t.Index));
        sel.TrackSort = TrackSort.Duration;
        Assert.Equal([1UL, 0, 2], sel.DisplayedTracks.Select(t => t.Index)); // 1000, 300, 100
    }

    [Fact]
    public void AlbumSort_ByYearDescending()
    {
        var engine = new FakeEngine();
        engine.AlbumList.AddRange([
            FakeEngine.Card(0) with { Name = "A", Year = "1999" },
            FakeEngine.Card(1) with { Name = "B", Year = "2010" },
            FakeEngine.Card(2) with { Name = "C", Year = "" }]);
        var library = new LibraryState { Engine = engine };
        library.Reload();
        var sel = new LibraryFilterState(library) { AlbumSort = AlbumSort.Year, SortAscending = false };

        Assert.Equal(["B", "A", "C"], sel.DisplayedAlbums.Select(a => a.Name));
    }

    [Fact]
    public void AlbumQuery_FiltersNameAndArtist_DiacriticInsensitive()
    {
        var engine = new FakeEngine();
        engine.AlbumList.AddRange([
            FakeEngine.Card(0) with { Name = "Discovery", Artist = "Daft Punk" },
            FakeEngine.Card(1) with { Name = "Melody AM", Artist = "Röyksopp" }]);
        var library = new LibraryState { Engine = engine };
        library.Reload();
        var sel = new LibraryFilterState(library);

        Assert.Equal(2, sel.DisplayedAlbums.Count);
        sel.Query = "royk";
        Assert.Equal([1UL], sel.DisplayedAlbums.Select(a => a.Index));
        sel.Query = "DISCO";
        Assert.Equal([0UL], sel.DisplayedAlbums.Select(a => a.Index));
    }

    [Fact]
    public void Select_FromAlbumGrid_SwitchesToTracks()
    {
        var (sel, _, _) = Create();
        sel.Mode = LibraryMode.Albums;
        sel.Select(0);
        Assert.Equal(LibraryMode.Tracks, sel.Mode);
    }

    [Fact]
    public void PlayTrack_QueuesSortedUnfilteredList_StartingAtTrack()
    {
        var sel = WithTracks(
            FakeEngine.Track(0) with { Title = "B" },
            FakeEngine.Track(1) with { Title = "C" },
            FakeEngine.Track(2) with { Title = "A" });
        var engine = (FakeEngine)sel.Engine!;
        sel.TrackSort = TrackSort.Title;
        sel.Query = "C";

        sel.PlayTrack(sel.DisplayedTracks[0]);

        Assert.Equal(["tracks:2,0,1@2"], engine.Calls);
    }

    [Fact]
    public void PlaySelected_PlaysAlbumInGridOrder_ByLibraryIndex()
    {
        // Album rows carry album-local indices; play resolves them by path.
        var sel = WithTracks(FakeEngine.Track(0), FakeEngine.Track(1), FakeEngine.Track(2));
        var engine = (FakeEngine)sel.Engine!;
        engine.AlbumDetail = _ => new AlbumDetailViewModel(FakeEngine.Card(0), 0, "0:00", "s",
            [FakeEngine.Track(2) with { Index = 0 }, FakeEngine.Track(1) with { Index = 1 }]);
        sel.Select(0);

        sel.PlaySelected();

        Assert.Equal(["tracks:2,1@0"], engine.Calls);
    }

    [Fact]
    public void AlbumCommands_InertWithoutSelection()
    {
        var (sel, _, engine) = Create();
        sel.PlaySelected();
        sel.QueueSelected();
        sel.ShuffleSelected();
        Assert.Empty(engine.Calls);
    }

    [Fact]
    public void Query_FiltersCaseInsensitive_AndPlaysUnfilteredRow()
    {
        var (sel, _, engine) = Create();
        sel.Query = "t1";
        Assert.Single(sel.DisplayedTracks);
        sel.PlayTrack(sel.DisplayedTracks[0]);
        Assert.Contains("tracks:0,1@1", engine.Calls);

        sel.Query = "al"; // album field
        Assert.Equal(2, sel.DisplayedTracks.Count);
        sel.Query = "zz";
        Assert.Empty(sel.DisplayedTracks);
        sel.Query = "  ";
        Assert.Equal(2, sel.DisplayedTracks.Count);
    }

    [Fact]
    public void Reload_WithoutIndex_ClearsDetail()
    {
        var (sel, _, _) = Create();
        sel.Reload();
        Assert.Null(sel.Detail);
    }
}
