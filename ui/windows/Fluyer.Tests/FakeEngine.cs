using Fluyer.Core.Native;
using Fluyer.Core.Support;

namespace Fluyer.Tests;

/// <summary>
/// Configurable <see cref="IFluyerEngine"/> double with a call log.
/// List-backed for library reads; delegates for snapshots and detail
/// (null detail simulates a failed fetch).
/// </summary>
internal sealed class FakeEngine : IFluyerEngine
{
    public Func<PlayerBarViewModel> BarView { get; set; } = () => ViewModelDefaults.NoTrack;
    public Func<PlayViewModel> PlayView { get; set; } = () => ViewModelDefaults.EmptyPlayView;
    public Func<ulong, AlbumDetailViewModel?> AlbumDetail { get; set; } = _ => null;
    public Func<ulong> Position { get; set; } = () => 0;
    public Func<ulong, int> LyricIndex { get; set; } = _ => -1;

    public List<TrackItemViewModel> TrackList { get; } = new();
    public List<AlbumCardViewModel> AlbumList { get; } = new();

    public List<string> Calls { get; } = new();
    public List<ulong> SeekTargets { get; } = new();
    public List<float> VolumeSets { get; } = new();
    public List<string[]> Scanned { get; } = new();

    public void Dispose() { }

    public PlayerBarViewModel GetPlayerBarView() => BarView();
    public PlayViewModel GetPlayView() => PlayView();
    public ulong GetTrackCount() => (ulong)TrackList.Count;
    public ulong GetAlbumCount() => (ulong)AlbumList.Count;

    public TrackItemViewModel? GetTrackView(ulong index)
        => index < (ulong)TrackList.Count ? TrackList[(int)index] : null;

    public AlbumCardViewModel? GetAlbumCard(ulong index)
        => index < (ulong)AlbumList.Count ? AlbumList[(int)index] : null;

    public AlbumDetailViewModel? GetAlbumDetail(ulong index) => AlbumDetail(index);
    public ulong GetPosition() => Position();
    public int GetActiveLyricIndex(ulong positionMs) => LyricIndex(positionMs);
    public float GetVolume() => 1.0f;

    public void Play() => Calls.Add("play");
    public void Pause() => Calls.Add("pause");
    public void TogglePlay() => Calls.Add("toggle");
    public void Next() => Calls.Add("next");
    public void Previous() => Calls.Add("previous");
    public void Seek(ulong positionMs) { Calls.Add("seek"); SeekTargets.Add(positionMs); }
    public void SetVolume(float volume) { Calls.Add("volume"); VolumeSets.Add(volume); }
    public void CycleRepeat() => Calls.Add("repeat");
    public void Shuffle() => Calls.Add("shuffle");
    public void RequestSync() => Calls.Add("sync");
    public void ScanDirectories(string[] directories) => Scanned.Add(directories);
    public void PlaySingleFromLibrary(ulong index) => Calls.Add($"single:{index}");
    public void PlayAllFromLibrary(ulong startIndex) => Calls.Add($"all:{startIndex}");
    public void PlayAlbum(ulong index) => Calls.Add($"album:{index}");
    public void PlayAlbumTrack(ulong albumIndex, ulong trackIndex) => Calls.Add($"albumtrack:{albumIndex}:{trackIndex}");
    public void QueueAlbum(ulong index) => Calls.Add($"queue:{index}");
    public void ShuffleAlbum(ulong index) => Calls.Add($"shufflealbum:{index}");

    public byte[]? GetTrackThumbnail(ulong index, uint maxSize) => null;
    public byte[]? GetAlbumThumbnail(ulong index, uint maxSize) => null;
    public byte[]? GetCurrentThumbnail(uint maxSize) => null;
    public (byte[] Rgba, uint Width, uint Height)? GenerateBackground(uint width, uint height) => null;

    internal static TrackItemViewModel Track(ulong index, bool current = false) => new(
        Index: index, Path: $"/music/{index}.mp3", Title: $"T{index}", Artist: "A",
        Album: "AL", DurationMs: 180_000, DurationFormatted: "3:00", IsCurrent: current);

    internal static AlbumCardViewModel Card(ulong index) => new(
        Index: index, Name: $"Album {index}", Artist: "A", Year: "2024",
        TrackCount: 10, TrackCountLabel: "10 tracks");
}
