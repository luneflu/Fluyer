using System.Runtime.InteropServices;
using Fluyer.Core.Native;
using Fluyer.Core.State;
using Xunit.Abstractions;

namespace Fluyer.Tests;

/// <summary>
/// Boots the real <c>fluyer_core.dll</c> + BASS from <c>target/debug</c>.
/// Skips (never fails) when the native binaries are absent — e.g. a source
/// checkout that has not run <c>cargo build -p fluyer_core</c> yet.
/// </summary>
public sealed class EngineSmokeTests
{
    [DllImport("kernel32", SetLastError = true, CharSet = CharSet.Unicode)]
    private static extern bool SetDllDirectory(string lpPathName);

    private static string? FindRepoRoot()
    {
        var dir = new DirectoryInfo(AppContext.BaseDirectory);
        while (dir is not null)
        {
            if (File.Exists(Path.Combine(dir.FullName, "Cargo.toml"))
                && Directory.Exists(Path.Combine(dir.FullName, "crates", "fluyer_core")))
            {
                return dir.FullName;
            }
            dir = dir.Parent;
        }
        return null;
    }

    private readonly ITestOutputHelper _output;

    public EngineSmokeTests(ITestOutputHelper output)
    {
        _output = output;
    }

    [Fact]
    public void RealEngine_BootsAndServesDefaults()
    {
        var root = FindRepoRoot();
        Assert.True(root is not null, "Could not locate repo root from test directory.");
        var nativeDir = Path.Combine(root!, "target", "debug");
        if (!File.Exists(Path.Combine(nativeDir, "fluyer_core.dll")))
        {
            _output.WriteLine("fluyer_core.dll not built; run cargo build -p fluyer_core first.");
            return;
        }
        SetDllDirectory(nativeDir);

        var dataDir = Path.Combine(Path.GetTempPath(), "fluyer-smoke", "data");
        var cacheDir = Path.Combine(Path.GetTempPath(), "fluyer-smoke", "cache");
        var sink = new NullSink();
        using var engine = new FluyerEngine(dataDir, cacheDir, sink);
        if (!engine.IsRunning)
        {
            _output.WriteLine("Core did not start (no audio device?).");
            return;
        }

        var bar = engine.GetPlayerBarView();
        Assert.Equal("No Track", bar.Title);
        Assert.Equal(0UL, engine.GetTrackCount());
        _output.WriteLine($"SMOKE-OK title={bar.Title} volume={bar.Volume} repeat={bar.RepeatMode}");
        // Empty-dir scan must not throw or wipe anything.
        engine.ScanDirectories([Path.GetTempPath()]);
        Assert.Equal(0UL, engine.GetTrackCount());
        Assert.Null(engine.GetTrackThumbnail(0, 88));
        Assert.Null(engine.GetCurrentThumbnail(400));
    }

    private sealed class NullSink : IFluyerEventSink
    {
        public void OnPlayerSync() { }
        public void OnTrackChanged(ulong index) { }
        public void OnScanProgress(ulong current, ulong total) { }
        public void OnToast(string message) { }
        public void OnTrackCoverLoaded(ulong index) { }
        public void OnAlbumCoverLoaded(ulong index) { }
        public void OnLyricsLoaded(string lyrics) { }
    }
}
