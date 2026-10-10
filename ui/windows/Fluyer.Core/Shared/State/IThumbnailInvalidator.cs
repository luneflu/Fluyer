namespace Fluyer.Core.State;

/// <summary>
/// Thumbnail-cache invalidation hook. Implemented by the app's image store;
/// kept as an interface so <see cref="AppState"/> stays UI-toolkit-free and
/// unit-testable.
/// </summary>
public interface IThumbnailInvalidator
{
    void InvalidatePrefix(string prefix);
}
