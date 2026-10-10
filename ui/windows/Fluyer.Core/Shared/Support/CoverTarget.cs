namespace Fluyer.Core.Support;

/// <summary>
/// Testable slice of the (key, token) stale-guard used by <c>CoverImages</c>.
/// Extracted so the recycle semantics — duplicate suppression, same-token
/// no-op, cross-key acceptance — are pinned by unit tests instead of only by
/// eyeballing the scrolling grid. Kept logic-free on purpose: the only state
/// is the current target of one image.
/// </summary>
public sealed class CoverTarget
{
    private string? _key;
    private object? _token;

    /// <summary>True when work must be issued; false when the request is a duplicate.</summary>
    public bool Prepare(string key, object token)
    {
        if (_key == key && ReferenceEquals(_token, token))
        {
            return false;
        }
        _key = key;
        _token = token;
        return true;
    }

    /// <summary>True when the completed load still owns the target.</summary>
    public bool Accept(string key, object token)
        => _key == key && ReferenceEquals(_token, token);
}
