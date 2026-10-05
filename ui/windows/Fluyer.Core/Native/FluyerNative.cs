using System.Runtime.InteropServices;

namespace Fluyer.Core.Native;

/// <summary>
/// Mirrors <c>FluyerRepeatMode</c> in <c>crates/fluyer_core/src/ffi.rs</c>.
/// Byte-backed to match Rust's <c>#[repr(u8)]</c>.
/// </summary>
internal enum FluyerRepeatMode : byte
{
    None = 0,
    All = 1,
    One = 2,
}

/// <summary>
/// Mirrors <c>FluyerPlayerState</c> in <c>ffi.rs</c>.
/// Rust <c>bool</c> is one byte — <see cref="UnmanagedType.U1"/> is required;
/// the default 4-byte marshaling would misalign every field after <c>Index</c>.
/// </summary>
[StructLayout(LayoutKind.Sequential)]
internal struct FluyerPlayerState
{
    public long Index;
    public ulong PositionMs;
    public ulong DurationMs;
    [MarshalAs(UnmanagedType.U1)]
    public bool IsPlaying;
    public FluyerRepeatMode RepeatMode;
    [MarshalAs(UnmanagedType.U1)]
    public bool IsShuffled;
}

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate void OnStateChanged(IntPtr userData, FluyerPlayerState state);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate void OnTrackChanged(IntPtr userData, IntPtr trackJson, ulong index);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate void OnScanProgress(IntPtr userData, ulong current, ulong total);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate void OnToast(IntPtr userData, IntPtr message);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate void OnCoverLoaded(IntPtr userData, ulong index);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate void OnLyricsLoaded(IntPtr userData, IntPtr lyrics);

/// <summary>
/// Mirrors <c>FluyerCallbacks</c> in <c>ffi.rs</c>. Function pointers are held as
/// <see cref="IntPtr"/> (via <c>Marshal.GetFunctionPointerForDelegate</c>) so the
/// managed delegates can be kept alive in fields on <see cref="FluyerEngine"/>
/// for the engine's whole lifetime.
/// </summary>
[StructLayout(LayoutKind.Sequential)]
internal struct FluyerCallbacks
{
    public IntPtr UserData;
    public IntPtr OnStateChanged;
    public IntPtr OnTrackChanged;
    public IntPtr OnScanProgress;
    public IntPtr OnToast;
    public IntPtr OnTrackCoverLoaded;
    public IntPtr OnAlbumCoverLoaded;
    public IntPtr OnLyricsLoaded;
}

/// <summary>
/// Raw P/Invoke surface over <c>fluyer_core.dll</c> (C ABI in
/// <c>crates/fluyer_core/src/ffi.rs</c>). All string returns are Rust-owned
/// <c>CString</c> pointers freed with <c>fluyer_string_free</c>; all byte
/// returns are Rust-owned buffers freed with <c>fluyer_bytes_free</c>.
/// </summary>
internal static class FluyerNative
{
    private const string Lib = "fluyer_core";

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr fluyer_init(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string dataDir,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string cacheDir,
        ref FluyerCallbacks callbacks);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_free(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_player_play(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_player_pause(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_player_toggle_play(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_player_next(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_player_previous(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_player_seek(IntPtr engine, ulong positionMs);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_player_set_volume(IntPtr engine, float volume);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern float fluyer_player_get_volume(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_player_set_repeat(IntPtr engine, FluyerRepeatMode mode);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_player_cycle_repeat(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_player_shuffle(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern ulong fluyer_player_get_position(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern int fluyer_player_get_active_lyric_index(IntPtr engine, ulong positionMs);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_player_request_sync(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_library_scan(IntPtr engine, IntPtr paths, ulong count);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern ulong fluyer_library_get_count(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern ulong fluyer_library_get_album_count(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr fluyer_library_get_track_view_json(IntPtr engine, ulong index);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr fluyer_library_get_album_view_json(IntPtr engine, ulong index);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr fluyer_library_get_album_detail_json(IntPtr engine, ulong index);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr fluyer_player_get_bar_view_json(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr fluyer_player_get_play_view_json(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_library_play_index(IntPtr engine, ulong index);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_library_play_all_from_index(IntPtr engine, ulong index);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_library_play_album(IntPtr engine, ulong index);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_library_play_album_track(IntPtr engine, ulong albumIndex, ulong trackIndex);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_library_queue_album(IntPtr engine, ulong index);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_library_shuffle_album(IntPtr engine, ulong index);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr fluyer_queue_get_json(IntPtr engine);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_queue_goto(IntPtr engine, ulong index);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_queue_remove(IntPtr engine, ulong index);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_queue_move(IntPtr engine, ulong from, ulong to);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr fluyer_library_get_track_thumbnail(
        IntPtr engine, ulong index, uint maxSize, out ulong outLen);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr fluyer_library_get_album_thumbnail(
        IntPtr engine, ulong index, uint maxSize, out ulong outLen);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr fluyer_player_get_current_thumbnail(
        IntPtr engine, uint maxSize, out ulong outLen);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr fluyer_player_generate_background(
        IntPtr engine, uint width, uint height,
        out uint outW, out uint outH, out ulong outLen);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr fluyer_format_time(ulong ms);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_bytes_free(IntPtr ptr, ulong len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    public static extern void fluyer_string_free(IntPtr s);
}
