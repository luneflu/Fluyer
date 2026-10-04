using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;

namespace Fluyer.Support;

/// <summary>
/// Async cover loading into list/card images with a stale-guard: the
/// <c>Image.Tag</c> carries the (key, item) pair, and a slow load from a
/// recycled container never overwrites the new item — the XAML equivalent of
/// the SwiftUI <c>.task(id:)</c> cancellation + <c>Task.isCancelled</c> guard.
/// </summary>
public static class CoverImages
{
    public static Task LoadInto(Image target, string key, object token, Func<byte[]?> load)
    {
        // Already showing (or already loading) this exact item: no-op. This
        // also collapses the duplicate refresh passes the grid/carousel fire
        // per selection change into a single load.
        if (target.Tag is (string k, object t) && k == key && ReferenceEquals(t, token))
        {
            return Task.CompletedTask;
        }
        var store = ThumbnailStore.Shared;
        if (store.Peek(key) is { } hit)
        {
            target.Tag = (key, token);
            target.Source = hit;
            target.Opacity = 1;
            return Task.CompletedTask;
        }

        // Miss: keep the old bitmap on screen until the new one arrives.
        // Clearing Source/Opacity here blanks every realized row at once and
        // the backdrop flashes through the gap — that blank-then-pop is the
        // flicker. A briefly stale cover on a recycled row is far cheaper
        // than a transparent hole; fresh rows already sit at Opacity 0 from XAML.
        target.Tag = (key, token);
        return LoadAsync(target, key, token, load);
    }

    private static async Task LoadAsync(Image target, string key, object token, Func<byte[]?> load)
    {
        var image = await ThumbnailStore.Shared.GetAsync(key, load).ConfigureAwait(true);
        // Recycled since? Drop the result.
        if (target.Tag is not (string k, object t) || k != key || !ReferenceEquals(t, token))
        {
            return;
        }
        if (image is null)
        {
            // No art for this key — fall back to the placeholder glyph.
            target.Source = null;
            target.Opacity = 0;
            return;
        }
        target.Source = image;
        target.Opacity = 1;
    }

    public static T? FindChild<T>(DependencyObject parent) where T : DependencyObject
    {
        var count = VisualTreeHelper.GetChildrenCount(parent);
        for (var i = 0; i < count; i++)
        {
            var child = VisualTreeHelper.GetChild(parent, i);
            if (child is T typed)
            {
                return typed;
            }
            if (FindChild<T>(child) is { } nested)
            {
                return nested;
            }
        }
        return null;
    }

    public static IEnumerable<T> FindChildren<T>(DependencyObject parent) where T : DependencyObject
    {
        var count = VisualTreeHelper.GetChildrenCount(parent);
        for (var i = 0; i < count; i++)
        {
            var child = VisualTreeHelper.GetChild(parent, i);
            if (child is T typed)
            {
                yield return typed;
            }
            foreach (var nested in FindChildren<T>(child))
            {
                yield return nested;
            }
        }
    }
}
