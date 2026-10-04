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
        var store = ThumbnailStore.Shared;
        if (store.Peek(key) is { } hit)
        {
            target.Tag = (key, token);
            target.Source = hit;
            target.Opacity = 1;
            return Task.CompletedTask;
        }

        target.Tag = (key, token);
        target.Source = null;
        target.Opacity = 0;
        return LoadAsync(target, key, token, load);
    }

    private static async Task LoadAsync(Image target, string key, object token, Func<byte[]?> load)
    {
        var image = await ThumbnailStore.Shared.GetAsync(key, load).ConfigureAwait(true);
        if (image is null)
        {
            return;
        }
        // Recycled since? Drop the result.
        if (target.Tag is not (string k, object t) || k != key || !ReferenceEquals(t, token))
        {
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
