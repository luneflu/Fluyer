using Fluyer.Core.Support;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;

namespace Fluyer.Support;

/// <summary>
/// Async cover loading into list/card images with a stale-guard: the
/// <c>Image.Tag</c> carries the (key, item) pair, and a slow load from a
/// recycled container never overwrites the new item — the XAML equivalent of
/// the SwiftUI <c>.task(id:)</c> cancellation + <c>Task.isCancelled</c> guard.
/// The per-image state machine is <see cref="CoverTarget"/> (unit-tested);
/// this class only wires it to UI elements.
/// </summary>
public static class CoverImages
{
    public static Task LoadInto(Image target, string key, object token, Func<Task<byte[]?>> load)
    {
        // Already showing (or already loading) this exact item: no-op. This
        // also collapses the duplicate refresh passes the grid/carousel fire
        // per selection change into a single load.
        var guard = GuardFor(target);
        bool start;
        BitmapImage? hit;
        lock (guard)
        {
            start = guard.Prepare(key, token);
            // Peek inside the same lock as Prepare: without this, N album rows
            // sharing one key all miss between another row's Store and their
            // own Peek and fire N duplicate fetches.
            hit = start ? ThumbnailStore.Shared.Peek(key) : null;
            if (hit is not null)
            {
                // Warm now — but LoadAsync may already be in flight for this
                // key/token; Accept (below) decides which completion wins.
            }
        }
        if (!start)
        {
            return Task.CompletedTask;
        }
        if (hit is not null)
        {
            target.Source = hit;
            target.Opacity = 1;
            return Task.CompletedTask;
        }

        // Miss: keep the old bitmap on screen until the new one arrives.
        // Clearing Source/Opacity here blanks every realized row at once and
        // the backdrop flashes through the gap — that blank-then-pop is the
        // flicker. A briefly stale cover on a recycled row is far cheaper
        // than a transparent hole; fresh rows already sit at Opacity 0 from XAML.
        return LoadAsync(target, key, token, load, guard);
    }

    private static async Task LoadAsync(Image target, string key, object token, Func<Task<byte[]?>> load, CoverTarget guard)
    {
        // Double-checked under the guard lock: a sibling row may have warmed
        // the cache between our Peek above and the fetch starting.
        if (ThumbnailStore.Shared.Peek(key) is { } warm)
        {
            lock (guard)
            {
                if (guard.Accept(key, token))
                {
                    target.Source = warm;
                    target.Opacity = 1;
                }
            }
            return;
        }
        var image = await ThumbnailStore.Shared.GetAsync(key, load).ConfigureAwait(true);
        lock (guard)
        {
            // Recycled since? Drop the result.
            if (!guard.Accept(key, token))
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
    }

    private static readonly DependencyProperty GuardProperty =
        DependencyProperty.RegisterAttached(
            "Guard", typeof(CoverTarget), typeof(CoverImages), new PropertyMetadata(null));

    // Tag is user-visible (tests, automation); track the guard in an attached
    // property so it can never collide with placeholders or test doubles.
    private static CoverTarget GuardFor(Image target)
        => (target.GetValue(GuardProperty) as CoverTarget)
            ?? (CoverTarget)SetGuard(target);

    private static object SetGuard(Image target)
    {
        var guard = new CoverTarget();
        target.SetValue(GuardProperty, guard);
        return guard;
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
