using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Core.Support;

namespace Fluyer.Tests;

public sealed class LibraryStateTests
{
    [Fact]
    public void Reload_PopulatesTracksAndAlbums()
    {
        var engine = new FakeEngine();
        engine.TrackList.AddRange([FakeEngine.Track(0), FakeEngine.Track(1)]);
        engine.AlbumList.Add(FakeEngine.Card(0));
        var library = new LibraryState { Engine = engine };
        library.Reload();
        Assert.Equal(2, library.Tracks.Count);
        Assert.Single(library.Albums);
        Assert.Equal(ViewModelDefaults.Idle, library.ScanStatus);
    }

    [Fact]
    public void ReloadActiveFlags_OnlyTouchesIsCurrent()
    {
        var engine = new FakeEngine();
        engine.TrackList.AddRange([FakeEngine.Track(0), FakeEngine.Track(1, current: true)]);
        var library = new LibraryState { Engine = engine };
        library.Reload();

        // Core flips the current marker to track 0.
        engine.TrackList[0] = FakeEngine.Track(0, current: true);
        engine.TrackList[1] = FakeEngine.Track(1, current: false);
        library.ReloadActiveFlags();

        Assert.True(library.Tracks[0].IsCurrent);
        Assert.False(library.Tracks[1].IsCurrent);
        Assert.Equal("T0", library.Tracks[0].Title);
    }

    [Fact]
    public void ReloadActiveFlags_CountMismatch_DoesNothing()
    {
        var engine = new FakeEngine();
        engine.TrackList.Add(FakeEngine.Track(0));
        var library = new LibraryState { Engine = engine };
        library.Reload();
        engine.TrackList.Add(FakeEngine.Track(1)); // rescan added a track
        library.ReloadActiveFlags(); // must not rebuild or throw
        Assert.Single(library.Tracks);
    }

    [Fact]
    public void NullEngine_IsInert()
    {
        var library = new LibraryState { Engine = null };
        library.Reload();
        library.ReloadActiveFlags();
        Assert.Empty(library.Tracks);
    }
}

public sealed class AlbumSelectionTests
{
    private static (AlbumSelection Sel, LibraryState Lib, FakeEngine Engine) Create()
    {
        var engine = new FakeEngine();
        engine.TrackList.AddRange([FakeEngine.Track(0), FakeEngine.Track(1)]);
        var library = new LibraryState { Engine = engine };
        library.Reload();
        var sel = new AlbumSelection(library) { Engine = engine };
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
    public void Reload_WithoutIndex_ClearsDetail()
    {
        var (sel, _, _) = Create();
        sel.Reload();
        Assert.Null(sel.Detail);
    }
}

public sealed class ToastStateTests
{
    [Fact]
    public void Show_SetsMessage()
    {
        var toast = new ToastState();
        toast.Show("hello");
        Assert.Equal("hello", toast.Message);
        toast.Dismiss();
    }

    [Fact]
    public void Replace_Supersedes()
    {
        var toast = new ToastState();
        toast.Show("first");
        toast.Show("second");
        Assert.Equal("second", toast.Message);
        toast.Dismiss();
    }

    [Fact]
    public void Dismiss_Clears()
    {
        var toast = new ToastState();
        toast.Show("hello");
        toast.Dismiss();
        Assert.Null(toast.Message);
    }

    [Fact]
    public async Task RapidToasts_NoPileup()
    {
        var toast = new ToastState();
        for (var i = 0; i < 50; i++)
        {
            toast.Show($"m{i}");
        }
        Assert.Equal("m49", toast.Message);
        toast.Dismiss();
        await Task.Delay(50);
        Assert.Null(toast.Message);
    }
}
