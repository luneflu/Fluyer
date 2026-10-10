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

    [Fact]
    public void PlayTrackAtRow_NegativeRow_Guarded()
    {
        var (sel, _, engine) = Create();
        sel.PlayTrackAtRow(-1);
        Assert.Empty(engine.Calls);
    }

    [Fact]
    public void PlayTrackAtRow_RoutesBySelection()
    {
        var (sel, _, engine) = Create();
        sel.PlayTrackAtRow(1);
        Assert.Contains("all:1", engine.Calls);

        engine.AlbumDetail = _ => new AlbumDetailViewModel(
            FakeEngine.Card(0), 0, "0:00", "s", []);
        sel.Select(0);
        sel.PlayTrackAtRow(2);
        Assert.Contains("albumtrack:0:2", engine.Calls);
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
        Assert.Contains("all:1", engine.Calls);

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
