namespace Fluyer.Core.Support;

/// <summary>
/// Single-flight + bounded-parallelism for cover fetches, over pure bytes.
/// Split out of the WinUI thumbnail store so the orchestration is unit-
/// testable: <c>BitmapImage</c> needs a UI thread, but dedup, gating and
/// invalidation do not. Thread-safe; never throws — load faults surface
/// as <c>null</c> so list rows degrade to placeholders.
/// </summary>
public sealed class ThumbnailFetchCoordinator
{
    private readonly SemaphoreSlim _gate;
    private readonly object _sync = new();
    private readonly Dictionary<string, Task<byte[]?>> _inflight = new();
    private readonly Dictionary<string, int> _versions = new();

    public ThumbnailFetchCoordinator(int maxParallel = 4)
    {
        var size = Math.Max(1, maxParallel);
        _gate = new SemaphoreSlim(size, size);
    }

    /// <summary>
    /// Bytes for <paramref name="key"/>, sharing one fetch across concurrent
    /// callers. <c>null</c> when the load fails, yields nothing, or the key
    /// was invalidated mid-flight.
    /// </summary>
    public Task<byte[]?> FetchAsync(string key, Func<Task<byte[]?>> load)
    {
        lock (_sync)
        {
            if (_inflight.TryGetValue(key, out var pending))
            {
                return pending;
            }
            var version = _versions.GetValueOrDefault(key);
            var tcs = new TaskCompletionSource<byte[]?>(TaskCreationOptions.RunContinuationsAsynchronously);
            // Placeholder first: RunAsync hits its first await (the gate) and
            // yields without running user code, so the real task is always
            // registered before any continuation can complete it.
            _inflight[key] = tcs.Task;
            _ = RunAsync(key, load, version, tcs);
            return tcs.Task;
        }
    }

    /// <summary>
    /// Drop in-flight fetches under <paramref name="prefix"/>; their
    /// continuations resolve <c>null</c> and the next fetch reloads.
    /// </summary>
    public void InvalidatePrefix(string prefix)
    {
        lock (_sync)
        {
            var doomed = _inflight.Keys
                .Where(k => k.StartsWith(prefix, StringComparison.Ordinal))
                .ToList();
            foreach (var key in doomed)
            {
                _inflight.Remove(key);
                _versions[key] = _versions.GetValueOrDefault(key) + 1;
            }
        }
    }

    private async Task RunAsync(
        string key, Func<Task<byte[]?>> load, int version, TaskCompletionSource<byte[]?> tcs)
    {
        // Yield once so the placeholder FetchAsync registered stays visible to
        // concurrent callers: without this, a synchronously-completing loader
        // (Task.FromResult — the unit-test shape) runs to completion inside
        // FetchAsync's lock and every caller fires its own load. Real loaders
        // are Task.Run (always async) so this costs one context switch.
        await Task.Yield();
        byte[]? result = null;
        try
        {
            await _gate.WaitAsync().ConfigureAwait(false);
            try
            {
                lock (_sync)
                {
                    // Invalidated while queued, or a newer fetch already owns
                    // this key: skip the FFI work.
                    if (_versions.GetValueOrDefault(key) != version
                        || !_inflight.TryGetValue(key, out var owner)
                        || !ReferenceEquals(owner, tcs.Task))
                    {
                        return;
                    }
                }
                var bytes = await load().ConfigureAwait(false);
                lock (_sync)
                {
                    // Invalidated, or superseded while running: drop stale pixels.
                    if (_versions.GetValueOrDefault(key) != version
                        || !_inflight.TryGetValue(key, out var owner)
                        || !ReferenceEquals(owner, tcs.Task))
                    {
                        return;
                    }
                    result = bytes;
                }
            }
            finally
            {
                _gate.Release();
            }
        }
        catch
        {
            result = null;
        }
        finally
        {
            lock (_sync)
            {
                // Only drop our own entry — a newer fetch may have replaced it.
                if (_inflight.TryGetValue(key, out var current) && ReferenceEquals(current, tcs.Task))
                {
                    _inflight.Remove(key);
                }
            }
            tcs.TrySetResult(result);
        }
    }
}
