using Fluyer.Core.State;

namespace Fluyer.Tests;

public sealed class SettingsStateTests : IDisposable
{
    private readonly string _dir = Path.Combine(Path.GetTempPath(), "fluyer-settings-" + Guid.NewGuid().ToString("N"));
    private string FilePath => Path.Combine(_dir, SettingsState.FileName);

    public void Dispose()
    {
        if (Directory.Exists(_dir))
        {
            Directory.Delete(_dir, recursive: true);
        }
    }

    [Fact]
    public void RoundTrips_AllFields()
    {
        var a = new SettingsState(FilePath);
        a.AddFolders(["C:\\Music", "D:\\Songs\\"]);
        a.AnimatedBackground = false;
        a.DiscordRpc = false;
        a.Volume = 0.3f;

        var b = new SettingsState(FilePath);
        Assert.Equal(new[] { "C:\\Music", "D:\\Songs" }, b.MusicFolders);
        Assert.False(b.AnimatedBackground);
        Assert.False(b.DiscordRpc);
        Assert.Equal(0.3f, b.Volume);
    }

    [Fact]
    public void AddFolders_DedupesCaseAndTrailingSeparator_KeepsDriveRoot()
    {
        var s = new SettingsState();
        var added = s.AddFolders(["C:\\Music", "c:\\music\\", "E:\\", "  "]);
        Assert.Equal(new[] { "C:\\Music", "E:\\" }, added);
        Assert.Empty(s.AddFolders(["C:\\MUSIC"]));
        Assert.Equal("C:\\Music", s.RemoveFolder("c:\\music\\"));
        Assert.Null(s.RemoveFolder("C:\\Music"));
        Assert.Equal(new[] { "E:\\" }, s.MusicFolders);
    }

    [Fact]
    public void CorruptFile_UsesDefaults_AndKeepsBackup()
    {
        Directory.CreateDirectory(_dir);
        File.WriteAllText(FilePath, "{ not json");

        var s = new SettingsState(FilePath);
        Assert.Empty(s.MusicFolders);
        Assert.True(s.AnimatedBackground);
        Assert.True(s.DiscordRpc);
        Assert.Equal(1.0f, s.Volume);
        Assert.Equal("{ not json", File.ReadAllText(FilePath + ".bad"));
    }

    [Fact]
    public void MissingFields_FallBackToDefaults()
    {
        Directory.CreateDirectory(_dir);
        File.WriteAllText(FilePath, "{\"MusicFolders\":[\"C:\\\\M\"]}");

        var s = new SettingsState(FilePath);
        Assert.Equal(new[] { "C:\\M" }, s.MusicFolders);
        Assert.True(s.AnimatedBackground);
        Assert.True(s.DiscordRpc);
        Assert.Equal(1.0f, s.Volume);
    }
}
