using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Components;

/// <summary>
/// Fades list items that sit under an open sidebar so the acrylic card only
/// shows the backdrop. Port of Sidebar.svelte + useAlbumList/useMusicList
/// <c>shouldHide*Item</c>: hidden items fade out and stop taking pointer input.
/// The visible band is window X: [lo, hi]; closed sidebars use ±infinity.
/// </summary>
public static class SidebarOcclusion
{
    // Svelte extraToleranceWidth: an item may poke this far under a pane and stay visible.
    private const double Tolerance = 10;

    // Svelte animation-duration: 500ms.
    private static readonly TimeSpan Fade = TimeSpan.FromMilliseconds(500);

#if DEBUG
    static SidebarOcclusion()
    {
        var inf = double.PositiveInfinity;
        System.Diagnostics.Debug.Assert(!IsCovered(0, 100, -inf, inf));
        System.Diagnostics.Debug.Assert(IsCovered(300, 400, 320, inf));     // starts under left pane
        System.Diagnostics.Debug.Assert(!IsCovered(315, 400, 320, inf));    // within tolerance
        System.Diagnostics.Debug.Assert(IsCovered(600, 720, -inf, 700));    // ends under right pane
        System.Diagnostics.Debug.Assert(!IsCovered(600, 705, -inf, 700));
    }
#endif

    public static bool IsCovered(double left, double right, double lo, double hi)
        => left < lo - Tolerance || right > hi + Tolerance;

    public static void Apply(UIElement container, bool covered)
    {
        container.OpacityTransition ??= new ScalarTransition { Duration = Fade };
        container.Opacity = covered ? 0 : 1;
        container.IsHitTestVisible = !covered;
    }

    /// <summary>Re-applies coverage to every realized container.</summary>
    public static void ApplyAll(ListViewBase list, Func<int, bool> covered)
    {
        if (list.ItemsPanelRoot is not Panel panel)
        {
            return;
        }
        foreach (var child in panel.Children)
        {
            var index = list.IndexFromContainer(child);
            if (index >= 0)
            {
                Apply(child, covered(index));
            }
        }
    }

    /// <summary>Element's left edge in window coordinates.</summary>
    public static double WindowX(UIElement element)
        => element.TransformToVisual(null).TransformPoint(default).X;
}
