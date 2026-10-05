using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Core.Support;

namespace Fluyer.Tests;

internal sealed class RecordingInvalidator : IThumbnailInvalidator
{
    public List<string> Prefixes { get; } = new();
    public void InvalidatePrefix(string prefix) => Prefixes.Add(prefix);
}

public sealed class AppStateTests
{
    private static (AppState App, FakeEngine Engine, RecordingInvalidator Thumbs) Create()
    {
        var engine = new FakeEngine();
        var thumbs = new RecordingInvalidator();
        var app = new AppState(engine, thumbs);
        return (app, engine, thumbs);
    }

    [Fact]
    public void Constructor_RefreshesAll()
    {
        var engine = new FakeEngine();
        engine.TrackList.Add(FakeEngine.Track(0));
        engine.AlbumList.Add(FakeEngine.Card(0));
        var app = new AppState(engine);
        Assert.Single(app.Library.Tracks);
        Assert.Single(app.Library.Albums);
    }

    [Fact]
    public void OnToast_ShowsMessage()
    {
        var (app, _, _) = Create();
        app.OnToast("hello");
        Assert.Equal("hello", app.Toast.Message);
        app.Toast.Dismiss();
    }

    [Fact]
    public void OnToast_ScanCompleted_RefreshesLibrary()
    {
        var (app, engine, _) = Create();
        engine.TrackList.Add(FakeEngine.Track(0));
        Assert.Empty(app.Library.Tracks);
        app.OnToast(FluyerEngine.ScanCompletedToast);
        Assert.Single(app.Library.Tracks);
        app.Toast.Dismiss();
    }

    [Fact]
    public void OnScanProgress_SetsStatus()
    {
        var (app, _, _) = Create();
        app.OnScanProgress(3, 10);
        Assert.True(app.Library.ScanStatus.IsScanning);
        Assert.Equal("Scanning: 3 / 10 files", app.Library.ScanStatus.StatusLabel);
    }

    [Fact]
    public void OnTrackChanged_ReloadsBarAndFlags()
    {
        var (app, engine, _) = Create();
        engine.TrackList.AddRange([FakeEngine.Track(0), FakeEngine.Track(1)]);
        engine.BarView = () => ViewModelDefaults.NoTrack with { Title = "Now" };
        app.OnTrackChanged(1);
        Assert.Equal("Now", app.Playback.Bar.Title);
    }

    [Fact]
    public void OnTrackCoverLoaded_InvalidatesAndReloadsPlayView()
    {
        var (app, engine, thumbs) = Create();
        engine.PlayView = () => ViewModelDefaults.EmptyPlayView with
        {
            Lyrics = [new LyricLine(0, "la")],
        };
        app.OnTrackCoverLoaded(4);
        Assert.Equal([ThumbnailKey.TrackPrefix(4)], thumbs.Prefixes);
        Assert.Single(app.Playback.PlayView.Lyrics);
    }

    [Fact]
    public void OnAlbumCoverLoaded_InvalidatesAlbumPrefix()
    {
        var (app, _, thumbs) = Create();
        app.OnAlbumCoverLoaded(2);
        Assert.Equal([ThumbnailKey.AlbumPrefix(2)], thumbs.Prefixes);
    }

    [Fact]
    public void ScanFolders_EmptyArray_DoesNothing()
    {
        var (app, engine, _) = Create();
        app.ScanFolders([]);
        Assert.Empty(engine.Scanned);
    }

    [Fact]
    public void ScanFolders_RemembersFolder_AndForwardsToCore()
    {
        var (app, engine, _) = Create();
        app.ScanFolders(["C:\\Music"]);
        Assert.Equal(new[] { "C:\\Music" }, Assert.Single(engine.Scanned));
        Assert.Equal(new[] { "C:\\Music" }, app.Settings.MusicFolders);
    }

    [Fact]
    public void Attach_RestoresSettings_AndRescansSavedFolders()
    {
        var settings = new SettingsState();
        settings.AddFolders(["C:\\A", "D:\\B"]);
        settings.DiscordRpc = false;
        settings.Volume = 0.25f;
        var engine = new FakeEngine();

        var app = new AppState(engine, settings: settings);

        Assert.Equal(new[] { "C:\\A", "D:\\B" }, Assert.Single(engine.Scanned));
        Assert.Contains("discord:False", engine.Calls);
        Assert.Equal(new[] { 0.25f }, engine.VolumeSets);
        Assert.Equal(0.25f, app.Playback.Bar.Volume);
    }

    [Fact]
    public void RemoveFolder_OnlyKnownFolder_HitsCore()
    {
        var (app, engine, _) = Create();
        app.ScanFolders(["C:\\Music"]);

        app.RemoveFolder("C:\\Other");
        app.RemoveFolder("c:\\music\\");

        Assert.Equal(new[] { "removefolder:C:\\Music" }, engine.Calls.Where(c => c.StartsWith("removefolder")));
        Assert.Empty(app.Settings.MusicFolders);
    }

    [Fact]
    public void DiscordToggle_ForwardsToCore()
    {
        var (app, engine, _) = Create();
        app.Settings.DiscordRpc = false;
        Assert.Equal("discord:False", engine.Calls.Last());
    }
}
