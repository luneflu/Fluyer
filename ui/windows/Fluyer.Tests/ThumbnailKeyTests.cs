using Fluyer.Core.Support;

namespace Fluyer.Tests;

public sealed class ThumbnailKeyTests
{
    [Fact]
    public void Size_IsPartOfKey_NoCarouselRowCollision()
        => Assert.NotEqual(ThumbnailKey.Album(3, 88), ThumbnailKey.Album(3, 400));

    [Fact]
    public void TrackAndAlbum_NamespacesDoNotCollide()
        => Assert.NotEqual(ThumbnailKey.Track(3, 88), ThumbnailKey.Album(3, 88));

    [Fact]
    public void Current_KeyedByPath_NotIndex()
    {
        Assert.Equal("current-200-/a/b.mp3", ThumbnailKey.Current(200, "/a/b.mp3"));
        Assert.NotEqual(
            ThumbnailKey.Current(200, "/a/b.mp3"),
            ThumbnailKey.Current(200, "/a/c.mp3"));
    }

    [Fact]
    public void Prefixes_CoverEverySize()
    {
        Assert.StartsWith(ThumbnailKey.TrackPrefix(7), ThumbnailKey.Track(7, 88));
        Assert.StartsWith(ThumbnailKey.TrackPrefix(7), ThumbnailKey.Track(7, 400));
        Assert.StartsWith(ThumbnailKey.AlbumPrefix(7), ThumbnailKey.Album(7, 88));
        Assert.False(ThumbnailKey.Track(70, 88).StartsWith(ThumbnailKey.TrackPrefix(7)));
    }
}
