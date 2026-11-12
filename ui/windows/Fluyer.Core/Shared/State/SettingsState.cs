using Fluyer.Core.Native;

namespace Fluyer.Core.State;

/// <summary>
/// User settings for the window. The core owns the file
/// (<c>%AppData%/org.alvindimas05.fluyer/settings.json</c>) and every rule that used
/// to live here: JSON shape, folder normalization, dedupe, volume clamp, unknown
/// backdrop value. This type is the observable mirror views bind to; it never touches
/// disk. A null engine keeps everything in memory (tests, headless).
/// </summary>
public sealed class SettingsState : Support.ObservableObject
{
    private readonly List<string> _folders = new();
    private bool _animatedBackground = true;
    private bool _discordRpc = true;
    private float _volume = 1.0f;
    private BackdropSource _backdropSource = BackdropSource.Artwork;
    private bool _applying;

    /// <summary>Set once by <see cref="AppState"/> right after the core starts.</summary>
    public IFluyerEngine? Engine { get; set; }

    public IReadOnlyList<string> MusicFolders => _folders.ToList();

    public bool AnimatedBackground
    {
        get => _animatedBackground;
        set
        {
            if (SetProperty(ref _animatedBackground, value))
            {
                Save();
            }
        }
    }

    public BackdropSource BackdropSource
    {
        get => _backdropSource;
        set
        {
            if (SetProperty(ref _backdropSource, value))
            {
                Save();
            }
        }
    }

    public bool DiscordRpc
    {
        get => _discordRpc;
        set
        {
            if (SetProperty(ref _discordRpc, value))
            {
                Save();
            }
        }
    }

    /// <summary>Last session volume; written by the caller at shutdown, not per slider tick.</summary>
    public float Volume
    {
        get => _volume;
        set
        {
            if (SetProperty(ref _volume, value))
            {
                Save();
            }
        }
    }

    /// <summary>Reads the stored settings; a null engine keeps the defaults.</summary>
    public void Load()
    {
        if (Engine is null)
        {
            return;
        }
        Apply(Engine.GetSettings());
    }

    /// <summary>Adds folders not already present. The core normalizes and dedupes; the result is what got stored.</summary>
    public IReadOnlyList<string> AddFolders(IEnumerable<string> paths)
    {
        var before = new List<string>(_folders);
        _folders.AddRange(paths);
        OnPropertyChanged(nameof(MusicFolders));
        Save();
        return _folders.Where(f => !before.Contains(f)).ToList();
    }

    /// <summary>Forgets a folder; returns the stored spelling (what scans used), or null if unknown.</summary>
    public string? RemoveFolder(string path)
    {
        var key = MatchingForm(path);
        var stored = _folders.FirstOrDefault(f => MatchingForm(f) == key);
        if (stored is null)
        {
            return null;
        }
        _folders.Remove(stored);
        OnPropertyChanged(nameof(MusicFolders));
        Save();
        return stored;
    }

    /// <summary>Compares a picked path with a stored one: the same trailing-separator trim the core uses.</summary>
    private static string MatchingForm(string path)
    {
        var trimmed = path.Trim();
        if (trimmed.Length <= 1)
        {
            return trimmed;
        }
        var stripped = trimmed.TrimEnd('\\', '/');
        if (stripped.Length == 2 && stripped[1] == ':')
        {
            return stripped + "\\";
        }
        return stripped.Length == 0 ? "/" : stripped;
    }

    private void Save()
    {
        if (_applying || Engine is null)
        {
            return;
        }
        var stored = Engine.SetSettings(new SettingsViewModel(
            _folders.ToList(),
            _animatedBackground,
            _backdropSource.ToString().ToLowerInvariant(),
            _discordRpc,
            _volume));
        if (stored is not null)
        {
            Apply(stored);
        }
    }

    /// <summary>Mirrors the core's canonical values back into the observable properties.</summary>
    private void Apply(SettingsViewModel stored)
    {
        _applying = true;
        try
        {
            _folders.Clear();
            _folders.AddRange(stored.MusicFolders);
            _animatedBackground = stored.AnimatedBackground;
            _backdropSource = Enum.TryParse<BackdropSource>(stored.BackdropSource, ignoreCase: true, out var source)
                && Enum.IsDefined(source) ? source : BackdropSource.Artwork;
            _discordRpc = stored.DiscordRpc;
            _volume = Math.Clamp(stored.Volume, 0.0f, 1.0f);
        }
        finally
        {
            _applying = false;
        }
        OnPropertyChanged(nameof(MusicFolders));
        OnPropertyChanged(nameof(AnimatedBackground));
        OnPropertyChanged(nameof(BackdropSource));
        OnPropertyChanged(nameof(DiscordRpc));
        OnPropertyChanged(nameof(Volume));
    }
}
