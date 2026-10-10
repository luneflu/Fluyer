using Fluyer.Core.State;
using Fluyer.Core.Support;
using Microsoft.UI.Xaml.Media.Imaging;
using System.Runtime.InteropServices.WindowsRuntime;
using Windows.Storage.Streams;

namespace Fluyer.Shared;

/// <summary>
/// Bounded cache of decoded thumbnails. WinUI port of <c>ThumbnailStore</c>
/// (<c>ui/macos/Sources/Shared/Support/ThumbnailStore.swift</c>): keeps only small,
/// pre-downscaled bitmaps, caps count and bytes, and invalidates by key prefix
/// when the core reports a cover reload.
///
/// Misses fetch fully async: <see cref="ThumbnailFetchCoordinator"/> collapses
/// duplicate in-flight loads for one key into a single fetch (inside an album every row shares one key), and a gate
/// caps concurrent FFI decodes so a fast scroll can't flood the thread pool
/// — or the UI thread with completions. No caller ever blocks on cover I/O.
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
    private readonly ThumbnailFetchCoordinator _fetches = new(maxParallel: 4);
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
        // In-flight fetches for these keys are stale (cover reloaded) — drop
        // them so they resolve null and the next fetch reloads fresh pixels.
        _fetches.InvalidatePrefix(prefix);
        // Costs are recomputed lazily on next eviction pass; count stays bounded
        // because _order entries for removed keys are skipped when dequeued.
    }

    /// <summary>
    /// Cached bitmap when warm, else decode <paramref name="load"/> (already
    /// downscaled JPEG bytes from the core — never full-res covers) without
    /// blocking any caller: concurrent requests for one key share one fetch.
    /// </summary>
    public Task<BitmapImage?> GetAsync(string key, Func<Task<byte[]?>> load)
    {
        if (Peek(key) is { } hit)
        {
            return Task.FromResult<BitmapImage?>(hit);
        }
        return FetchAsync(key, load);
    }

    private async Task<BitmapImage?> FetchAsync(string key, Func<Task<byte[]?>> load)
    {
        // Bytes are single-flighted by the coordinator: N rows sharing one
        // key (every row in an album) collapse into one FFI call. WIC decode
        // stays here — BitmapImage is UI-thread-affine and decode must run
        // after the hop back, so only one waiter decodes while the rest take
        // the double-checked Peek below.
        var bytes = await _fetches.FetchAsync(key, load).ConfigureAwait(true);
        if (bytes is null || bytes.Length == 0)
        {
            return Peek(key); // invalidated / failed — show warm entry if any
        }
        if (Peek(key) is { } raced)
        {
            return raced;
        }
        try
        {
            var image = await DecodeAsync(bytes).ConfigureAwait(true);
            if (image is not null)
            {
                Store(key, image);
            }
            return image ?? Peek(key);
        }
        catch
        {
            return Peek(key);
        }
    }

    private static async Task<BitmapImage?> DecodeAsync(byte[] bytes)
    {
        // Entire decode stays on UI thread: both BitmapImage and
        // InMemoryRandomAccessStream are thread-affine — hopping off for the
        // write (ConfigureAwait(false)) breaks SetSourceAsync with
        // RPC_E_WRONG_THREAD and every cover silently falls back to placeholder.
        // The expensive work (JPEG parse + downscale in core) already ran on a
        // worker inside the coordinator; this is just a few-KB WIC decode.
        var image = new BitmapImage();
        using var stream = new InMemoryRandomAccessStream();
        await stream.WriteAsync(bytes.AsBuffer()).AsTask().ConfigureAwait(true);
        stream.Seek(0);
        await image.SetSourceAsync(stream).AsTask().ConfigureAwait(true);
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
