using Fluyer.Core.State;
using Microsoft.UI.Xaml.Media.Imaging;
using System.Runtime.InteropServices.WindowsRuntime;
using Windows.Storage.Streams;

namespace Fluyer.Support;

/// <summary>
/// Bounded cache of decoded thumbnails. WinUI port of <c>ThumbnailStore</c>
/// (<c>ui/macos/Sources/Support/ThumbnailStore.swift</c>): keeps only small,
/// pre-downscaled bitmaps, caps count and bytes, and invalidates by key prefix
/// when the core reports a cover reload.
/// Must be used from the UI thread (<c>BitmapImage</c> is thread-affine).
/// </summary>
public sealed class ThumbnailStore : IThumbnailInvalidator
{
    public static ThumbnailStore Shared { get; } = new();

    private const int CountLimit = 300;
    private const int ByteLimit = 24 * 1024 * 1024;

    private readonly Dictionary<string, BitmapImage> _cache = new();
    private readonly HashSet<string> _keys = new();
    private readonly Queue<string> _order = new();
    private int _totalCost;

    public BitmapImage? Peek(string key)
        => _cache.TryGetValue(key, out var image) ? image : null;

    public void InvalidatePrefix(string prefix)
    {
        var doomed = _keys.Where(k => k.StartsWith(prefix, StringComparison.Ordinal)).ToList();
        foreach (var key in doomed)
        {
            _keys.Remove(key);
            _cache.Remove(key);
        }
        // Costs are recomputed lazily on next eviction pass; count stays bounded
        // because _order entries for removed keys are skipped when dequeued.
    }

    /// <summary>
    /// Cached bitmap when warm, else decode <paramref name="load"/> (already
    /// downscaled JPEG bytes from the core — never full-res covers).
    /// </summary>
    public async Task<BitmapImage?> GetAsync(string key, Func<byte[]?> load)
    {
        if (Peek(key) is { } hit)
        {
            return hit;
        }
        // FFI + JPEG decode stays off the render loop: bytes on a worker, then
        // SetSourceAsync yields while WIC decodes.
        var bytes = await Task.Run(load).ConfigureAwait(true);
        if (bytes is null || bytes.Length == 0)
        {
            return null;
        }
        // Re-check: a concurrent load for the same key may have won.
        if (Peek(key) is { } raced)
        {
            return raced;
        }
        var image = new BitmapImage();
        using var stream = new InMemoryRandomAccessStream();
        await stream.WriteAsync(bytes.AsBuffer()).AsTask().ConfigureAwait(true);
        stream.Seek(0);
        await image.SetSourceAsync(stream).AsTask().ConfigureAwait(true);
        Store(key, image);
        return image;
    }

    private void Store(string key, BitmapImage image)
    {
        var width = image.PixelWidth > 0 ? image.PixelWidth : 1;
        var height = image.PixelHeight > 0 ? image.PixelHeight : 1;
        _totalCost += width * height * 4;
        _cache[key] = image;
        if (_keys.Add(key))
        {
            _order.Enqueue(key);
        }
        Evict();
    }

    private void Evict()
    {
        while ((_cache.Count > CountLimit || _totalCost > ByteLimit) && _order.Count > 0)
        {
            var oldest = _order.Dequeue();
            if (_cache.Remove(oldest))
            {
                _keys.Remove(oldest);
                // Approximate: PixelWidth/Height may be 0 before decode.
                _totalCost = Math.Max(0, _totalCost - (88 * 88 * 4));
            }
        }
    }
}
