using Fluyer.Core.Native;
using Fluyer.Core.Support;

namespace Fluyer.Core.State;

/// <summary>
/// Root of the app's observable state. Port of <c>AppState</c>
/// (<c>ui/macos/Sources/Shared/State/AppState.swift</c>): owns the engine, turns core
/// events into updates on the split state objects, and runs the cross-cutting
/// full-library refresh.
/// </summary>
public sealed class AppState : Support.ObservableObject, IFluyerEventSink
{
    public PlaybackState Playback { get; } = new();
    public LibraryState Library { get; } = new();
    public LibraryFilterState Selection { get; }
    public ToastState Toast { get; } = new();
    public QueueState Queue { get; } = new();
    public SettingsState Settings { get; }

    public IFluyerEngine? Engine { get; private set; }

    private readonly IThumbnailInvalidator? _thumbnails;

    public AppState(IFluyerEngine? engine, IThumbnailInvalidator? thumbnails = null, SettingsState? settings = null)
    {
        Selection = new LibraryFilterState(Library);
        _thumbnails = thumbnails;
        Settings = settings ?? new SettingsState();
        Settings.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(SettingsState.DiscordRpc))
            {
                Engine?.SetDiscordEnabled(Settings.DiscordRpc);
            }
        };

        AttachEngine(engine);
    }

    /// <summary>
    /// Late engine binding for hosts that must exist before the core can start
    /// (the core takes the state itself as its event sink).
    /// </summary>
    public void AttachEngine(IFluyerEngine? engine)
    {
        Engine = engine;
        Playback.Engine = engine;
        Library.Engine = engine;
        Selection.Engine = engine;
        Queue.Engine = engine;

        Refresh();
        if (engine is null)
        {
            return;
        }
        // Restore persisted settings, then re-scan saved folders so files
        // added/removed while closed show up (unchanged files are skipped by mtime).
        engine.SetDiscordEnabled(Settings.DiscordRpc);
        Playback.SetVolume(Settings.Volume);
        ScanSavedFolders();
    }

    /// <summary>Re-read everything the window shows from the core.</summary>
    public void Refresh()
    {
        Playback.ReloadBar();
        Playback.ReloadPlayView();
        Library.Reload();
        Selection.Reload();
        Queue.Reload();
    }

    /// <summary>Remember picked folders and scan them.</summary>
    public void ScanFolders(string[] paths)
    {
        if (paths.Length == 0)
        {
            return;
        }
        Settings.AddFolders(paths);
        Engine?.ScanDirectories(paths);
    }

    /// <summary>Re-scan every saved folder.</summary>
    public void ScanSavedFolders()
    {
        var folders = Settings.MusicFolders.ToArray();
        if (folders.Length > 0)
        {
            Engine?.ScanDirectories(folders);
        }
    }

    /// <summary>Forget a folder and drop its tracks from the library.</summary>
    public void RemoveFolder(string path)
    {
        // Forward the stored spelling: DB rows carry the scan root's exact casing.
        var stored = Settings.RemoveFolder(path);
        if (stored is null)
        {
            return;
        }
        Engine?.RemoveFolder(stored);
        Refresh();
    }

    /// <summary>Play the whole library from the first track (legacy menu "Play All").</summary>
    public void PlayAll()
    {
        if (Library.Tracks.Count > 0)
        {
            Engine?.PlayAllFromLibrary(0);
        }
    }

    /// <summary>Persist session state that changes too often to save live (volume).</summary>
    public void SaveSession() => Settings.Volume = Playback.Bar.Volume;

    // MARK: - IFluyerEventSink (already on the UI thread — FluyerEngine hops)

    public void OnPlayerSync()
    {
        Playback.ReloadBar();
        Queue.Reload();
    }

    public void OnTrackChanged(ulong index)
    {
        Playback.ApplyTrackChange();
        Library.ReloadActiveFlags();
        // Sorted or searched grids hold a snapshot; the default order is live already.
        if (Selection.DisplayedTracks is not System.Collections.ObjectModel.ObservableCollection<TrackItemViewModel>)
        {
            Selection.RefreshDisplayed();
        }
        Queue.Reload();
    }

    public void OnScanProgress(ulong current, ulong total)
        => Library.ScanStatus = ScanStatus.Scanning(current, total);

    public void OnToast(string message)
    {
        Toast.Show(message);
        // Mirror of the core's own fan-out (uniffi_api/events.rs emits LibraryUpdated
        // after this toast): the C ABI has no library-updated callback, so the
        // shell performs the same string check.
        if (message == FluyerEngine.ScanCompletedToast)
        {
            Refresh();
        }
    }

    public void OnTrackCoverLoaded(ulong index)
    {
        _thumbnails?.InvalidatePrefix(ThumbnailKey.TrackPrefix(index));
        Playback.ReloadPlayView();
    }

    public void OnAlbumCoverLoaded(ulong index)
        => _thumbnails?.InvalidatePrefix(ThumbnailKey.AlbumPrefix(index));

    public void OnLyricsLoaded(string lyrics) => Playback.ReloadPlayView();
}
