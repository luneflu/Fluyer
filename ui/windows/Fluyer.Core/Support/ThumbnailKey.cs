namespace Fluyer.Core.Support;

/// <summary>
/// Port of <c>ThumbnailKey</c> (<c>ui/macos/Sources/Support/ThumbnailKey.swift</c>).
/// Pixel size stays part of every key: the carousel (~400px) and track rows
/// (88px) share cover indexes and must not collide.
/// </summary>
public static class ThumbnailKey
{
    public static string Track(ulong index, int px) => $"track-{index}@{px}";

    public static string Album(ulong index, int px) => $"album-{index}@{px}";

    public static string Current(int side, string path) => $"current-{side}-{path}";

    public static string TrackPrefix(ulong index) => $"track-{index}@";

    public static string AlbumPrefix(ulong index) => $"album-{index}@";
}
