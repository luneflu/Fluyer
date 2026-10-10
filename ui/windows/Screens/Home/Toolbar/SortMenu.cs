using Fluyer.Core.State;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Screens;

/// <summary>
/// Toolbar sort control. Port of <c>SortMenu</c>
/// (<c>ui/macos/Sources/Screens/Home/Toolbar/SortMenu.swift</c>): keys follow the
/// current mode, the direction is shared. The flyout is rebuilt on open, so the
/// radio checks always reflect <see cref="LibraryFilterState"/>.
/// </summary>
public sealed partial class SortMenu : Button
{
    public static readonly DependencyProperty FilterProperty =
        DependencyProperty.Register(nameof(Filter), typeof(LibraryFilterState), typeof(SortMenu),
            new PropertyMetadata(null));

    public LibraryFilterState? Filter
    {
        get => (LibraryFilterState?)GetValue(FilterProperty);
        set => SetValue(FilterProperty, value);
    }

    private readonly MenuFlyout _menu = new() { Placement = Microsoft.UI.Xaml.Controls.Primitives.FlyoutPlacementMode.BottomEdgeAlignedRight };

    public SortMenu()
    {
        Content = new FontIcon { Glyph = "\uE8CB", FontSize = 14 };
        ToolTipService.SetToolTip(this, "Sort");
        Microsoft.UI.Xaml.Automation.AutomationProperties.SetName(this, "Sort");
        Flyout = _menu;
        _menu.Opening += (_, _) => Rebuild();
    }

    private void Rebuild()
    {
        _menu.Items.Clear();
        if (Filter is not { } filter)
        {
            return;
        }
        _menu.Items.Add(new MenuFlyoutItem { Text = "Sort By", IsEnabled = false });
        if (filter.Mode == LibraryMode.Albums)
        {
            AddGroup("key", Enum.GetValues<AlbumSort>(), filter.AlbumSort, v => filter.AlbumSort = v, Label);
        }
        else
        {
            AddGroup("key", Enum.GetValues<TrackSort>(), filter.TrackSort, v => filter.TrackSort = v, v => v.ToString());
        }
        _menu.Items.Add(new MenuFlyoutSeparator());
        _menu.Items.Add(new MenuFlyoutItem { Text = "Order", IsEnabled = false });
        AddGroup("order", [true, false], filter.SortAscending, v => filter.SortAscending = v,
            v => v ? "Ascending" : "Descending");
    }

    private void AddGroup<T>(string group, IEnumerable<T> values, T current, Action<T> pick, Func<T, string> label)
    {
        foreach (var value in values)
        {
            var item = new RadioMenuFlyoutItem
            {
                Text = label(value),
                GroupName = group,
                IsChecked = EqualityComparer<T>.Default.Equals(value, current),
            };
            item.Click += (_, _) => pick(value);
            _menu.Items.Add(item);
        }
    }

    private static string Label(AlbumSort sort) => sort == AlbumSort.TrackCount ? "Track Count" : sort.ToString();
}
