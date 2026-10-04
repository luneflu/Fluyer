using System.Runtime.InteropServices;
using System.Text.Json;

namespace Fluyer.Core.Native;

/// <summary>
/// Core-to-UI events. Mirrors the <c>FluyerEvent</c> fan-out in
/// <c>ui/macos/Sources/State/AppState.swift</c>: payloads that the C ABI does
/// not carry (player-bar metadata, play-view snapshot) are re-read from the
/// engine by the state layer instead of being passed through here.
/// </summary>
public interface IFluyerEventSink
{
    void OnPlayerSync();
    void OnTrackChanged(ulong index);
    void OnScanProgress(ulong current, ulong total);
    void OnToast(string message);
    void OnTrackCoverLoaded(ulong index);
    void OnAlbumCoverLoaded(ulong index);
    void OnLyricsLoaded(string lyrics);
}

/// <summary>
/// Engine surface consumed by the state layer. Deliberately narrow so tests
/// can substitute a fake; every method maps 1:1 to a <c>fluyer_*</c> export
/// (or a default where the core has no opinion, e.g. scan status idle).
/// </summary>
public interface IFluyerEngine : IDisposable
{
    PlayerBarViewModel GetPlayerBarView();
    PlayViewModel GetPlayView();
    ulong GetTrackCount();
    ulong GetAlbumCount();
    TrackItemViewModel? GetTrackView(ulong index);
    AlbumCardViewModel? GetAlbumCard(ulong index);
    AlbumDetailViewModel? GetAlbumDetail(ulong index);
    ulong GetPosition();
    int GetActiveLyricIndex(ulong positionMs);
    float GetVolume();

    void Play();
    void Pause();
    void TogglePlay();
    void Next();
    void Previous();
    void Seek(ulong positionMs);
    void SetVolume(float volume);
    void CycleRepeat();
    void Shuffle();
    void RequestSync();
    void ScanDirectories(string[] directories);
    void PlaySingleFromLibrary(ulong index);
    void PlayAllFromLibrary(ulong startIndex);
    void PlayAlbum(ulong index);
    void PlayAlbumTrack(ulong albumIndex, ulong trackIndex);
    void QueueAlbum(ulong index);
    void ShuffleAlbum(ulong index);

    public byte[]? GetTrackThumbnail(ulong index, uint maxSize);
    public byte[]? GetAlbumThumbnail(ulong index, uint maxSize);
    public byte[]? GetCurrentThumbnail(uint maxSize);
    public Task<byte[]?> GetTrackThumbnailAsync(ulong index, uint maxSize);
    public Task<byte[]?> GetAlbumThumbnailAsync(ulong index, uint maxSize);
    public Task<byte[]?> GetCurrentThumbnailAsync(uint maxSize);
    (byte[] Rgba, uint Width, uint Height)? GenerateBackground(uint width, uint height);
}

/// <summary>
/// Managed owner of the native <c>FluyerEngine</c>. Creates the on-disk layout
/// (<c>%AppData%/org.alvindimas05.fluyer</c> + local cache dir), registers
/// callbacks, and hops every callback onto the constructing thread's
/// <see cref="SynchronizationContext"/> (the UI thread in the app; direct
/// invocation when there is none, e.g. headless use).
/// </summary>
public sealed class FluyerEngine : IFluyerEngine
{
    /// <summary>Toast text after which the core's library contents changed.</summary>
    public const string ScanCompletedToast = "Library scan completed";

    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNameCaseInsensitive = false,
    };

    private IntPtr _handle;
    private readonly SynchronizationContext? _context;
    private readonly GCHandle _sinkHandle;

    // Kept alive for the engine's lifetime: the native side holds bare
    // function pointers with no GC knowledge.
    private readonly OnStateChanged _onStateChanged;
    private readonly OnTrackChanged _onTrackChanged;
    private readonly OnScanProgress _onScanProgress;
    private readonly OnToast _onToast;
    private readonly OnCoverLoaded _onTrackCoverLoaded;
    private readonly OnCoverLoaded _onAlbumCoverLoaded;
    private readonly OnLyricsLoaded _onLyricsLoaded;

    private bool _disposed;

    public bool IsRunning => _handle != IntPtr.Zero;

    public FluyerEngine(string dataDir, string cacheDir, IFluyerEventSink sink)
    {
        _context = SynchronizationContext.Current;

        _onStateChanged = HandleStateChanged;
        _onTrackChanged = HandleTrackChanged;
        _onScanProgress = HandleScanProgress;
        _onToast = HandleToast;
        _onTrackCoverLoaded = HandleTrackCoverLoaded;
        _onAlbumCoverLoaded = HandleAlbumCoverLoaded;
        _onLyricsLoaded = HandleLyricsLoaded;

        // Strong handle on the *sink*: callbacks only fire while the engine
        // lives, and the handle is freed only via Dispose below.
        _sinkHandle = GCHandle.Alloc(sink, GCHandleType.Normal);

        var callbacks = new FluyerCallbacks
        {
            UserData = GCHandle.ToIntPtr(_sinkHandle),
            OnStateChanged = Marshal.GetFunctionPointerForDelegate(_onStateChanged),
            OnTrackChanged = Marshal.GetFunctionPointerForDelegate(_onTrackChanged),
            OnScanProgress = Marshal.GetFunctionPointerForDelegate(_onScanProgress),
            OnToast = Marshal.GetFunctionPointerForDelegate(_onToast),
            OnTrackCoverLoaded = Marshal.GetFunctionPointerForDelegate(_onTrackCoverLoaded),
            OnAlbumCoverLoaded = Marshal.GetFunctionPointerForDelegate(_onAlbumCoverLoaded),
            OnLyricsLoaded = Marshal.GetFunctionPointerForDelegate(_onLyricsLoaded),
        };

        Directory.CreateDirectory(dataDir);
        Directory.CreateDirectory(cacheDir);

        _handle = FluyerNative.fluyer_init(dataDir, cacheDir, ref callbacks);
    }

    // MARK: - Queries

    public PlayerBarViewModel GetPlayerBarView()
        => DeserializeOrDefault(
            () => TakeString(FluyerNative.fluyer_player_get_bar_view_json(_handle)),
            Support.ViewModelDefaults.NoTrack);

    public PlayViewModel GetPlayView()
        => DeserializeOrDefault(
            () => TakeString(FluyerNative.fluyer_player_get_play_view_json(_handle)),
            Support.ViewModelDefaults.EmptyPlayView);

    public ulong GetTrackCount() => Check(() => FluyerNative.fluyer_library_get_count(_handle));

    public ulong GetAlbumCount() => Check(() => FluyerNative.fluyer_library_get_album_count(_handle));

    public TrackItemViewModel? GetTrackView(ulong index)
        => DeserializeNullable<TrackItemViewModel>(
            () => TakeString(FluyerNative.fluyer_library_get_track_view_json(_handle, index)));

    public AlbumCardViewModel? GetAlbumCard(ulong index)
        => DeserializeNullable<AlbumCardViewModel>(
            () => TakeString(FluyerNative.fluyer_library_get_album_view_json(_handle, index)));

    public AlbumDetailViewModel? GetAlbumDetail(ulong index)
        => DeserializeNullable<AlbumDetailViewModel>(
            () => TakeString(FluyerNative.fluyer_library_get_album_detail_json(_handle, index)));

    public ulong GetPosition() => Check(() => FluyerNative.fluyer_player_get_position(_handle));

    public int GetActiveLyricIndex(ulong positionMs)
        => Check(() => FluyerNative.fluyer_player_get_active_lyric_index(_handle, positionMs));

    public float GetVolume() => Check(() => FluyerNative.fluyer_player_get_volume(_handle));

    // MARK: - Commands

    public void Play() => Invoke(() => FluyerNative.fluyer_player_play(_handle));
    public void Pause() => Invoke(() => FluyerNative.fluyer_player_pause(_handle));
    public void TogglePlay() => Invoke(() => FluyerNative.fluyer_player_toggle_play(_handle));
    public void Next() => Invoke(() => FluyerNative.fluyer_player_next(_handle));
    public void Previous() => Invoke(() => FluyerNative.fluyer_player_previous(_handle));
    public void Seek(ulong positionMs) => Invoke(() => FluyerNative.fluyer_player_seek(_handle, positionMs));
    public void SetVolume(float volume) => Invoke(() => FluyerNative.fluyer_player_set_volume(_handle, volume));
    public void CycleRepeat() => Invoke(() => FluyerNative.fluyer_player_cycle_repeat(_handle));
    public void Shuffle() => Invoke(() => FluyerNative.fluyer_player_shuffle(_handle));
    public void RequestSync() => Invoke(() => FluyerNative.fluyer_player_request_sync(_handle));

    public void ScanDirectories(string[] directories)
    {
        if (_handle == IntPtr.Zero || directories.Length == 0)
        {
            return;
        }
        // Marshal as a native array of UTF-8 C strings.
        var pointers = new IntPtr[directories.Length];
        try
        {
            for (var i = 0; i < directories.Length; i++)
            {
                pointers[i] = Marshal.StringToCoTaskMemUTF8(directories[i]);
            }
            var array = Marshal.AllocCoTaskMem(IntPtr.Size * pointers.Length);
            try
            {
                Marshal.Copy(pointers, 0, array, pointers.Length);
                FluyerNative.fluyer_library_scan(_handle, array, (ulong)pointers.Length);
            }
            finally
            {
                Marshal.FreeCoTaskMem(array);
            }
        }
        finally
        {
            foreach (var p in pointers)
            {
                if (p != IntPtr.Zero)
                {
                    Marshal.FreeCoTaskMem(p);
                }
            }
        }
    }

    public void PlaySingleFromLibrary(ulong index)
        => Invoke(() => FluyerNative.fluyer_library_play_index(_handle, index));

    public void PlayAllFromLibrary(ulong startIndex)
        => Invoke(() => FluyerNative.fluyer_library_play_all_from_index(_handle, startIndex));

    public void PlayAlbum(ulong index)
        => Invoke(() => FluyerNative.fluyer_library_play_album(_handle, index));

    public void PlayAlbumTrack(ulong albumIndex, ulong trackIndex)
        => Invoke(() => FluyerNative.fluyer_library_play_album_track(_handle, albumIndex, trackIndex));

    public void QueueAlbum(ulong index)
        => Invoke(() => FluyerNative.fluyer_library_queue_album(_handle, index));

    public void ShuffleAlbum(ulong index)
        => Invoke(() => FluyerNative.fluyer_library_shuffle_album(_handle, index));

    // MARK: - Images

    // Sync thumbnails decode on the calling thread (JPEG parse + Triangle
    // downscale + re-encode). List rows must use the Async twins below, which
    // hop the FFI call onto a worker — calling these from a container callback
    // stalls layout and drops frames.
    public byte[]? GetTrackThumbnail(ulong index, uint maxSize)
    {
        if (_handle == IntPtr.Zero)
        {
            return null;
        }
        var ptr = FluyerNative.fluyer_library_get_track_thumbnail(_handle, index, maxSize, out var len);
        return TakeBytes(ptr, len);
    }

    public byte[]? GetAlbumThumbnail(ulong index, uint maxSize)
    {
        if (_handle == IntPtr.Zero)
        {
            return null;
        }
        var ptr = FluyerNative.fluyer_library_get_album_thumbnail(_handle, index, maxSize, out var len);
        return TakeBytes(ptr, len);
    }

    public byte[]? GetCurrentThumbnail(uint maxSize)
    {
        if (_handle == IntPtr.Zero)
        {
            return null;
        }
        var ptr = FluyerNative.fluyer_player_get_current_thumbnail(_handle, maxSize, out var len);
        return TakeBytes(ptr, len);
    }

    public Task<byte[]?> GetTrackThumbnailAsync(ulong index, uint maxSize)
        => Task.Run(() => GetTrackThumbnail(index, maxSize));

    public Task<byte[]?> GetAlbumThumbnailAsync(ulong index, uint maxSize)
        => Task.Run(() => GetAlbumThumbnail(index, maxSize));

    public Task<byte[]?> GetCurrentThumbnailAsync(uint maxSize)
        => Task.Run(() => GetCurrentThumbnail(maxSize));

    public (byte[] Rgba, uint Width, uint Height)? GenerateBackground(uint width, uint height)
    {
        if (_handle == IntPtr.Zero)
        {
            return null;
        }
        var ptr = FluyerNative.fluyer_player_generate_background(
            _handle, width, height, out var w, out var h, out var len);
        if (ptr == IntPtr.Zero || len == 0)
        {
            return null;
        }
        try
        {
            var bytes = new byte[len];
            Marshal.Copy(ptr, bytes, 0, checked((int)len));
            return (bytes, w, h);
        }
        finally
        {
            FluyerNative.fluyer_bytes_free(ptr, len);
        }
    }

    // MARK: - Callbacks (fire on arbitrary Rust threads — hop to _context)

    private void Dispatch(Action action)
    {
        if (_context is null)
        {
            action();
        }
        else
        {
            _context.Post(_ => action(), null);
        }
    }

    private void HandleStateChanged(IntPtr userData, FluyerPlayerState state)
        => Dispatch(() => Sink(userData).OnPlayerSync());

    private void HandleTrackChanged(IntPtr userData, IntPtr trackJson, ulong index)
        => Dispatch(() => Sink(userData).OnTrackChanged(index));

    private void HandleScanProgress(IntPtr userData, ulong current, ulong total)
        => Dispatch(() => Sink(userData).OnScanProgress(current, total));

    private void HandleToast(IntPtr userData, IntPtr message)
    {
        var text = Marshal.PtrToStringUTF8(message) ?? string.Empty;
        Dispatch(() => Sink(userData).OnToast(text));
    }

    private void HandleTrackCoverLoaded(IntPtr userData, ulong index)
        => Dispatch(() => Sink(userData).OnTrackCoverLoaded(index));

    private void HandleAlbumCoverLoaded(IntPtr userData, ulong index)
        => Dispatch(() => Sink(userData).OnAlbumCoverLoaded(index));

    private void HandleLyricsLoaded(IntPtr userData, IntPtr lyrics)
    {
        var text = Marshal.PtrToStringUTF8(lyrics) ?? string.Empty;
        Dispatch(() => Sink(userData).OnLyricsLoaded(text));
    }

    private static IFluyerEventSink Sink(IntPtr userData)
    {
        var handle = GCHandle.FromIntPtr(userData);
        return (IFluyerEventSink)handle.Target!;
    }

    // MARK: - Marshalling helpers

    private void ThrowIfDead()
    {
        if (_handle == IntPtr.Zero)
        {
            throw new InvalidOperationException("Fluyer core failed to start.");
        }
    }

    private void Invoke(Action native)
    {
        if (_handle == IntPtr.Zero)
        {
            return;
        }
        native();
    }

    private T Check<T>(Func<T> native)
    {
        ThrowIfDead();
        return native();
    }

    private static string? TakeString(IntPtr ptr)
    {
        if (ptr == IntPtr.Zero)
        {
            return null;
        }
        try
        {
            return Marshal.PtrToStringUTF8(ptr);
        }
        finally
        {
            FluyerNative.fluyer_string_free(ptr);
        }
    }

    private static byte[]? TakeBytes(IntPtr ptr, ulong len)
    {
        if (ptr == IntPtr.Zero || len == 0)
        {
            return null;
        }
        try
        {
            var bytes = new byte[len];
            Marshal.Copy(ptr, bytes, 0, checked((int)len));
            return bytes;
        }
        finally
        {
            FluyerNative.fluyer_bytes_free(ptr, len);
        }
    }

    private static T DeserializeOrDefault<T>(Func<string?> take, T fallback) where T : class
        => DeserializeNullable<T>(take) ?? fallback;

    private static T? DeserializeNullable<T>(Func<string?> take) where T : class
    {
        var json = take();
        if (string.IsNullOrEmpty(json))
        {
            return null;
        }
        try
        {
            return JsonSerializer.Deserialize<T>(json, JsonOptions);
        }
        catch (JsonException)
        {
            return null;
        }
    }

    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }
        _disposed = true;
        if (_handle != IntPtr.Zero)
        {
            FluyerNative.fluyer_free(_handle);
            _handle = IntPtr.Zero;
        }
        if (_sinkHandle.IsAllocated)
        {
            _sinkHandle.Free();
        }
        GC.SuppressFinalize(this);
    }

    ~FluyerEngine() => Dispose();
}
