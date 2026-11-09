using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media.Animation;
using Microsoft.UI.Xaml.Shapes;

namespace Fluyer.Shared;

/// <summary>
/// <c>AlbumCard.swift</c> selected/dimmed/hover chrome. Swift keeps this in
/// per-card <c>@State</c>; WinUI recycles containers, so the views find the
/// named chrome borders and re-apply on filter, hover and recycle.
/// </summary>
public static class AlbumCardChrome
{
    /// <summary>Selected gets a solid white border; the rest dim to 40% while an album is open.</summary>
    public static void Apply(FrameworkElement card, bool selected, bool dimmed)
    {
        card.OpacityTransition = new ScalarTransition
        {
            Duration = TimeSpan.FromSeconds(Layout.Number("AlbumsDimFade")),
        };
        card.Opacity = dimmed ? 0.4 : 1;
        if (Find(card, "SelectedChrome") is { } selectedChrome)
        {
            selectedChrome.Opacity = selected ? 1 : 0;
        }
        if (selected && Find(card, "HoverChrome") is { } hover)
        {
            hover.Opacity = 0;
        }
    }

    /// <summary>Hover wash + border; a selected card keeps its solid border instead.</summary>
    public static void Hover(FrameworkElement card, bool hovering, bool selected)
    {
        if (selected || Find(card, "HoverChrome") is not { } chrome)
        {
            return;
        }
        var fade = new DoubleAnimation
        {
            To = hovering ? 1 : 0,
            Duration = new Duration(TimeSpan.FromSeconds(
                Layout.Number(hovering ? "AlbumsHoverFadeIn" : "AlbumsHoverFadeOut"))),
            EasingFunction = new CubicEase { EasingMode = EasingMode.EaseInOut },
        };
        Storyboard.SetTarget(fade, chrome);
        Storyboard.SetTargetProperty(fade, "Opacity");
        var storyboard = new Storyboard();
        storyboard.Children.Add(fade);
        storyboard.Begin();
    }

    /// <summary>Hides the music-note placeholder once real art paints; call again on recycle.</summary>
    public static void ShowPlaceholder(Image cover, bool show)
    {
        if (cover.Parent is not Panel panel)
        {
            return;
        }
        foreach (var child in panel.Children)
        {
            if (child is Rectangle or FontIcon)
            {
                child.Visibility = show ? Visibility.Visible : Visibility.Collapsed;
            }
        }
    }

    private static Border? Find(FrameworkElement card, string name)
        => CoverImages.FindChildren<Border>(card).FirstOrDefault(b => b.Name == name);
}
