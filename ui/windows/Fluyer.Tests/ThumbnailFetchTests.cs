using Fluyer.Core.Support;

namespace Fluyer.Tests;

public sealed class ThumbnailFetchCoordinatorTests
{
    [Fact]
    public async Task ConcurrentFetches_ShareOneLoad()
    {
        var coordinator = new ThumbnailFetchCoordinator(maxParallel: 4);
        var calls = 0;
        byte[] Payload()
        {
            Interlocked.Increment(ref calls);
            return [1, 2, 3];
        }
        var tasks = Enumerable.Range(0, 10)
            .Select(_ => coordinator.FetchAsync("album-0@88", () => Task.FromResult<byte[]?>(Payload())))
            .ToArray();
        var results = await Task.WhenAll(tasks);
        Assert.Equal(1, calls);
        Assert.All(results, r => Assert.Equal([1, 2, 3], r));
    }

    [Fact]
    public async Task LoadFailure_ResolvesNull_NeverThrows()
    {
        var coordinator = new ThumbnailFetchCoordinator();
        var r1 = await coordinator.FetchAsync("k", () => Task.FromResult<byte[]?>(null));
        var r2 = await coordinator.FetchAsync("k", () => Task.FromException<byte[]?>(new InvalidOperationException()));
        Assert.Null(r1);
        Assert.Null(r2);
    }

    [Fact]
    public async Task InvalidatePrefix_AbortsQueuedFetch_NextFetchReloads()
    {
        var coordinator = new ThumbnailFetchCoordinator(maxParallel: 1);
        var gate = new TaskCompletionSource<byte[]?>();
        // Occupy the single gate slot so the second fetch queues behind it.
        var blocker = coordinator.FetchAsync("other", () => gate.Task);
        var calls = 0;
        var queued = coordinator.FetchAsync("track-7@88", () =>
        {
            calls++;
            return Task.FromResult<byte[]?>([9]);
        });
        coordinator.InvalidatePrefix("track-7@");
        gate.SetResult([1, 1]);
        await blocker;
        Assert.Null(await queued); // stale — invalidated while queued
        var fresh = await coordinator.FetchAsync("track-7@88", () =>
        {
            calls++;
            return Task.FromResult<byte[]?>([7]);
        });
        Assert.Equal([7], fresh);
        Assert.Equal(1, calls); // only the reload ran the loader
    }

    [Fact]
    public async Task Gate_CapsParallelLoads()
    {
        const int maxParallel = 2;
        var coordinator = new ThumbnailFetchCoordinator(maxParallel);
        var concurrent = 0;
        var peak = 0;
        var release = new TaskCompletionSource();
        async Task<byte[]?> Load()
        {
            var now = Interlocked.Increment(ref concurrent);
            lock (this)
            {
                peak = Math.Max(peak, now);
            }
            await release.Task;
            Interlocked.Decrement(ref concurrent);
            return [5];
        }
        var tasks = Enumerable.Range(0, 5)
            .Select(i => coordinator.FetchAsync($"k{i}", Load))
            .ToArray();
        await Task.Delay(200); // let the gate fill — only 2 may be inside Load
        Assert.True(Volatile.Read(ref concurrent) <= maxParallel,
            $"gate leaked: {concurrent} concurrent loads");
        release.SetResult();
        var results = await Task.WhenAll(tasks);
        Assert.All(results, r => Assert.Equal([5], r));
        Assert.Equal(maxParallel, peak);
    }

    [Fact]
    public async Task AfterCompletion_NewFetch_Reloads()
    {
        var coordinator = new ThumbnailFetchCoordinator();
        var calls = 0;
        Task<byte[]?> Load() => Task.FromResult<byte[]?>([(byte)Interlocked.Increment(ref calls)]);
        Assert.Equal([1], await coordinator.FetchAsync("k", Load));
        // First fetch completed — the next one must run the loader again.
        Assert.Equal([2], await coordinator.FetchAsync("k", Load));
        Assert.Equal(2, calls);
    }
}

public sealed class CoverTargetTests
{
    [Fact]
    public void DuplicatePrepare_SuppressesSecondLoad()
    {
        var target = new CoverTarget();
        var token = new object();
        Assert.True(target.Prepare("album-0@88", token));
        Assert.False(target.Prepare("album-0@88", token)); // recycle pass — no-op
    }

    [Fact]
    public void SameToken_NewKey_Accepts()
    {
        var target = new CoverTarget();
        var token = new object();
        Assert.True(target.Prepare("a", token));
        Assert.True(target.Prepare("b", token)); // album switch — new key
        Assert.True(target.Accept("b", token));
        Assert.False(target.Accept("a", token)); // stale completion dropped
    }

    [Fact]
    public void SameKey_NewToken_Accepts()
    {
        var target = new CoverTarget();
        var t1 = new object();
        var t2 = new object();
        Assert.True(target.Prepare("a", t1));
        Assert.True(target.Prepare("a", t2)); // recycled to another row, same art
        Assert.True(target.Accept("a", t2));
        Assert.False(target.Accept("a", t1)); // old row's completion dropped
    }

    [Fact]
    public void UntouchedTarget_AcceptsNothing()
    {
        var target = new CoverTarget();
        Assert.False(target.Accept("a", new object()));
    }
}
