using Fluyer.Core.State;

namespace Fluyer.Tests;

/// <summary>
/// The file, the JSON shape, folder normalization and the volume clamp all live in
/// the core (<c>crates/fluyer_core/src/services/settings.rs</c> and its tests). These
/// cover what is left in C#: the defaults a view sees before the core answers,
/// remembering folders while no engine is attached, and matching a picked path.
/// </summary>
public sealed class SettingsStateTests
{
    [Fact]
    public void Defaults_MatchTheCoreDefaults()
    {
        var s = new SettingsState();
        Assert.Empty(s.MusicFolders);
        Assert.True(s.AnimatedBackground);
        Assert.True(s.DiscordRpc);
        Assert.Equal(1.0f, s.Volume);
        Assert.Equal(BackdropSource.Artwork, s.BackdropSource);
    }

    [Fact]
    public void AddFolders_WithoutAnEngine_RemembersAndReturnsTheNewOnes()
    {
        var s = new SettingsState();
        Assert.Equal(new[] { "C:\\Music", "D:\\Songs\\" }, s.AddFolders(["C:\\Music", "D:\\Songs\\"]));
        Assert.Equal(new[] { "C:\\Music", "D:\\Songs\\" }, s.MusicFolders);
        Assert.Empty(s.AddFolders(["C:\\Music"]));
    }

    [Fact]
    public void RemoveFolder_MatchesTrailingSeparatorAndDriveRoot_ReturnsStoredSpelling()
    {
        var s = new SettingsState();
        s.AddFolders(["C:\\Music", "E:\\"]);
        Assert.Equal("C:\\Music", s.RemoveFolder("C:\\Music\\"));
        Assert.Null(s.RemoveFolder("C:\\Music"));
        Assert.Equal("E:\\", s.RemoveFolder("E:"));
        Assert.Empty(s.MusicFolders);
    }

    [Fact]
    public void Load_WithoutAnEngine_KeepsTheDefaults()
    {
        var s = new SettingsState();
        s.Load();
        Assert.Empty(s.MusicFolders);
        Assert.True(s.AnimatedBackground);
    }
}
