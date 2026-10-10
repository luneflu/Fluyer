using System.Text.Json;

namespace Fluyer.Core.State;

/// <summary>
/// User settings persisted as JSON (legacy used <c>tauri-plugin-store</c>).
/// <c>null</c> path = in-memory only (tests, headless).
/// </summary>
public sealed class SettingsState : Support.ObservableObject
{
    public const string FileName = "settings.json";

    private sealed record Snapshot(
        List<string> MusicFolders,
        bool AnimatedBackground = true,
        bool DiscordRpc = true,
        float Volume = 1.0f);

    private static readonly JsonSerializerOptions JsonOptions = new() { WriteIndented = true };

    private readonly string? _path;
    private readonly List<string> _folders = new();
    private bool _animatedBackground = true;
    private bool _discordRpc = true;
    private float _volume = 1.0f;

    public SettingsState(string? path = null)
    {
        _path = path;
        Load();
    }

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
            if (SetProperty(ref _volume, Math.Clamp(value, 0.0f, 1.0f)))
            {
                Save();
            }
        }
    }

    /// <summary>Adds folders not already present (case-insensitive, trailing separator ignored). Returns the ones added.</summary>
    public IReadOnlyList<string> AddFolders(IEnumerable<string> paths)
    {
        var added = new List<string>();
        foreach (var raw in paths)
        {
            var path = Normalize(raw);
            if (path.Length > 0 && !_folders.Any(f => SamePath(f, path)))
            {
                _folders.Add(path);
                added.Add(path);
            }
        }
        if (added.Count > 0)
        {
            OnPropertyChanged(nameof(MusicFolders));
            Save();
        }
        return added;
    }

    /// <summary>Forgets a folder; returns the stored spelling (what scans used), or null if unknown.</summary>
    public string? RemoveFolder(string path)
    {
        var stored = _folders.FirstOrDefault(f => SamePath(f, Normalize(path)));
        if (stored is null)
        {
            return null;
        }
        _folders.Remove(stored);
        OnPropertyChanged(nameof(MusicFolders));
        Save();
        return stored;
    }

    private static string Normalize(string path)
    {
        var trimmed = path.Trim();
        // Keep drive roots ("C:\") intact; strip the separator elsewhere.
        return trimmed.Length > 3 ? trimmed.TrimEnd('\\', '/') : trimmed;
    }

    private static bool SamePath(string a, string b) => string.Equals(a, b, StringComparison.OrdinalIgnoreCase);

    private void Load()
    {
        if (_path is null || !File.Exists(_path))
        {
            return;
        }
        try
        {
            var snap = JsonSerializer.Deserialize<Snapshot>(File.ReadAllText(_path));
            if (snap is null)
            {
                return;
            }
            _folders.AddRange(snap.MusicFolders?.Select(Normalize).Where(p => p.Length > 0).Distinct(StringComparer.OrdinalIgnoreCase) ?? []);
            _animatedBackground = snap.AnimatedBackground;
            _discordRpc = snap.DiscordRpc;
            _volume = Math.Clamp(snap.Volume, 0.0f, 1.0f);
        }
        catch (Exception ex) when (ex is JsonException or IOException or UnauthorizedAccessException)
        {
            // Unreadable file: keep a copy so the next save doesn't destroy it.
            System.Diagnostics.Debug.WriteLine($"Settings unreadable, using defaults: {ex.Message}");
            try { File.Copy(_path, _path + ".bad", overwrite: true); } catch (IOException) { }
        }
    }

    private void Save()
    {
        if (_path is null)
        {
            return;
        }
        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(_path)!);
            var tmp = _path + ".tmp";
            File.WriteAllText(tmp, JsonSerializer.Serialize(
                new Snapshot(_folders.ToList(), _animatedBackground, _discordRpc, _volume), JsonOptions));
            File.Move(tmp, _path, overwrite: true);
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            System.Diagnostics.Debug.WriteLine($"Settings save failed: {ex.Message}");
        }
    }
}
