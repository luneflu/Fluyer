using Fluyer.Core.Native;
using Fluyer.Core.Support;

namespace Fluyer.Core.State;

/// <summary>
/// Creates and owns the core engine plus the on-disk layout it needs. Port of
/// <c>EngineHandle</c> (<c>ui/macos/Sources/State/EngineHandle.swift</c>).
/// </summary>
public static class EngineHandle
{
    /// <summary>
    /// App-data directory name. Hardcoded (not read from the package identity)
    /// so unpackaged/dev builds share storage with the installed app.
    /// </summary>
    public const string Identifier = "org.alvindimas05.fluyer";

    public static string DataDirectory => Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), Identifier);

    public static string CacheDirectory => Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), Identifier);

    /// <summary>
    /// Starts the core. Returns <c>null</c> (instead of throwing) when the
    /// native library cannot load, so the window still renders placeholders.
    /// </summary>
    public static IFluyerEngine? Attach(IFluyerEventSink sink)
    {
        try
        {
            var engine = new FluyerEngine(DataDirectory, CacheDirectory, sink);
            return engine.IsRunning ? engine : null;
        }
        catch (Exception ex) when (ex is DllNotFoundException
            or EntryPointNotFoundException
            or InvalidOperationException)
        {
            System.Diagnostics.Debug.WriteLine($"Fluyer core failed to start: {ex.Message}");
            return null;
        }
    }
}

/// <summary>
/// Thumbnail-cache invalidation hook. Implemented by the app's image store;
/// kept as an interface so <see cref="AppState"/> stays UI-toolkit-free and
/// unit-testable.
/// </summary>
public interface IThumbnailInvalidator
{
    void InvalidatePrefix(string prefix);
}

/// <summary>
/// Root of the app's observable state. Port of <c>AppState</c>
/// (<c>ui/macos/Sources/State/AppState.swift</c>): owns the engine, turns core
/// events into updates on the split state objects, and runs the cross-cutting
/// full-library refresh.
/// </summary>
public sealed class AppState : Support.ObservableObject, IFluyerEventSink
{
    public PlaybackState Playback { get; } = new();
    public LibraryState Library { get; } = new();
    public AlbumSelection Selection { get; }
    public ToastState Toast { get; } = new();

    public IFluyerEngine? Engine { get; private set; }

    private readonly IThumbnailInvalidator? _thumbnails;

    public AppState(IFluyerEngine? engine, IThumbnailInvalidator? thumbnails = null)
    {
        Selection = new AlbumSelection(Library);
        _thumbnails = thumbnails;

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

        Refresh();
    }

    /// <summary>Re-read everything the window shows from the core.</summary>
    public void Refresh()
    {
        Playback.ReloadBar();
        Playback.ReloadPlayView();
        Library.Reload();
        Selection.Reload();
    }

    /// <summary>Ask the core to scan folders. The UI picks them via FileOpenPicker first.</summary>
    public void ScanFolders(string[] paths)
    {
        if (paths.Length == 0)
        {
            return;
        }
        Engine?.ScanDirectories(paths);
    }

    // MARK: - IFluyerEventSink (already on the UI thread — FluyerEngine hops)

    public void OnPlayerSync() => Playback.ReloadBar();

    public void OnTrackChanged(ulong index)
    {
        Playback.ApplyTrackChange();
        Library.ReloadActiveFlags();
    }

    public void OnScanProgress(ulong current, ulong total)
        => Library.ScanStatus = ScanStatus.Scanning(current, total);

    public void OnToast(string message)
    {
        Toast.Show(message);
        // Mirror of the core's own fan-out (uniffi_api.rs emits LibraryUpdated
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
