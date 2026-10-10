using Fluyer.Core.Native;
using Fluyer.Core.Support;

namespace Fluyer.Core.State;

/// <summary>
/// Creates and owns the core engine plus the on-disk layout it needs. Port of
/// <c>EngineHandle</c> (<c>ui/macos/Sources/Shared/State/EngineHandle.swift</c>).
/// </summary>
public static class EngineHandle
{
    /// <summary>
    /// App-data directory name. Hardcoded (not read from the package identity)
    /// so unpackaged/dev builds share storage with the installed app.
    /// </summary>
    public const string Identifier = "org.alvindimas05.fluyer";

    public static string DataDirectory => Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), Identifier);

    public static string CacheDirectory => Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), Identifier);

    /// <summary>
    /// Starts the core. Returns <c>null</c> (instead of throwing) when the
    /// native library cannot load, so the window still renders placeholders.
    /// </summary>
    public static IFluyerEngine? Attach(IFluyerEventSink sink)
    {
        try
        {
            var engine = new FluyerEngine(DataDirectory, CacheDirectory, sink);
            return engine.IsRunning ? engine : null;
        }
        catch (Exception ex) when (ex is DllNotFoundException
            or EntryPointNotFoundException
            or InvalidOperationException)
        {
            System.Diagnostics.Debug.WriteLine($"Fluyer core failed to start: {ex.Message}");
            return null;
        }
    }
}
