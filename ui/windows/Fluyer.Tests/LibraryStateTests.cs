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
