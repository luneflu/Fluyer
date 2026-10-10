using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Core.Support;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Shared;

/// <summary>
/// Cover art for every view: track, album and now-playing thumbnails plus the
/// backdrop / media-overlay bytes. The only Windows UI code that asks the engine
/// for images; views call these and never touch the engine. Port of
/// <c>CoverState</c> (<c>ui/macos/Sources/Shared/State/CoverState.swift</c>).
/// Lives in the WinUI project (not Fluyer.Core) because it loads into
/// <see cref="Image"/> through the UI-thread-bound <see cref="ThumbnailStore"/>.
/// </summary>
public sealed class CoverState : IThumbnailInvalidator
{
    public IFluyerEngine? Engine { get; set; }

    public Task LoadTrack(Image target, ulong index, int px, object token)
    {
        var engine = Engine;
        return engine is null
            ? Task.CompletedTask
            : CoverImages.LoadInto(target, ThumbnailKey.Track(index, px), token,
                () => engine.GetTrackThumbnailAsync(index, (uint)px));
    }

    public Task LoadAlbum(Image target, ulong index, int px, object token)
    {
        var engine = Engine;
        return engine is null
            ? Task.CompletedTask
            : CoverImages.LoadInto(target, ThumbnailKey.Album(index, px), token,
                () => engine.GetAlbumThumbnailAsync(index, (uint)px));
    }

    /// <summary>Playing track's cover. <paramref name="path"/> is in the key so a track change never serves the previous song's bitmap.</summary>
    public Task LoadCurrent(Image target, int px, string path)
    {
        var engine = Engine;
        return engine is null
            ? Task.CompletedTask
            : CoverImages.LoadInto(target, ThumbnailKey.Current(px, path), path,
                () => engine.GetCurrentThumbnailAsync((uint)px));
    }

    /// <summary>Playing track's cover as JPEG bytes, uncached: one-off large uses (backdrop, media overlay) would only evict grid thumbnails.</summary>
    public Task<byte[]?> CurrentBytes(int px)
        => Engine?.GetCurrentThumbnailAsync((uint)px) ?? Task.FromResult<byte[]?>(null);

    public void InvalidatePrefix(string prefix) => ThumbnailStore.Shared.InvalidatePrefix(prefix);
}
