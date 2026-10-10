using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Core.Support;
using Fluyer.Components;
using Fluyer.Shared;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Layout = Fluyer.Shared.Layout;

namespace Fluyer.Screens;

/// <summary>
/// Track rows. Thumbnails load per realized container (the recycle-aware
/// equivalent of SwiftUI's <c>.task(id:)</c>); inside an album every row shares
/// the album cover key, otherwise each row uses its own track cover.
/// </summary>
public sealed partial class MusicGridView : UserControl
{
    public static readonly DependencyProperty LibraryProperty =
        DependencyProperty.Register(nameof(Library), typeof(LibraryState), typeof(MusicGridView),
            new PropertyMetadata(null, OnLibraryStateChanged));

    public static readonly DependencyProperty FilterProperty =
        DependencyProperty.Register(nameof(Filter), typeof(LibraryFilterState), typeof(MusicGridView),
            new PropertyMetadata(null, OnFilterStateChanged));

    public static readonly DependencyProperty CoversProperty =
        DependencyProperty.Register(nameof(Covers), typeof(CoverState), typeof(MusicGridView),
            new PropertyMetadata(null));

    public static readonly DependencyProperty CardItemWidthProperty =
        DependencyProperty.Register(nameof(CardItemWidth), typeof(double), typeof(MusicGridView),
            new PropertyMetadata(280.0));

    public LibraryState? Library
    {
        get => (LibraryState?)GetValue(LibraryProperty);
        set => SetValue(LibraryProperty, value);
    }

    public LibraryFilterState? Filter
    {
        get => (LibraryFilterState?)GetValue(FilterProperty);
        set => SetValue(FilterProperty, value);
    }

    /// <summary>Only image path to the engine; see <see cref="CoverState"/>.</summary>
    public CoverState? Covers
    {
        get => (CoverState?)GetValue(CoversProperty);
        set => SetValue(CoversProperty, value);
    }

    public double CardItemWidth
    {
        get => (double)GetValue(CardItemWidthProperty);
        private set => SetValue(CardItemWidthProperty, value);
    }

    /// <summary>Raised when the empty-state button needs to open settings (handled by the window).</summary>
    public event Action? OpenSettingsRequested;

    // x:Bind static function binding for the artist • album line.
    public static string ArtistAlbumLine(string artist, string album) => $"{artist} • {album}";

    // Read from Shared/Theme/Layout.xaml (TrackRow*, MusicGrid*).
    private static int Pixels => (int)Layout.Number("TrackRowCoverPixels");

    /// <summary>Gap between cards = item margin; the list's negative right margin cancels the last one.</summary>
    private static double Gap => Layout.Edges("MusicGridItemMargin").Right;

    private static double MinColumnWidth => Layout.Number("MusicGridMinColumnWidth");

    // Window-X band left visible by open sidebars (SidebarOcclusion).
    private double _bandLo = double.NegativeInfinity;
    private double _bandHi = double.PositiveInfinity;

    /// <summary>Fades columns outside window-X [lo, hi] (under an open sidebar).</summary>
    public void SetVisibleBand(double lo, double hi)
    {
        _bandLo = lo;
        _bandHi = hi;
        SidebarOcclusion.ApplyAll(Rows, IsCovered);
    }

    // Whole columns hide together, like useMusicList.isHiddenBySidebar.
    private bool IsCovered(int index)
    {
        if (double.IsNegativeInfinity(_bandLo) && double.IsPositiveInfinity(_bandHi))
        {
            return false;
        }
        var (cols, itemWidth) = ComputeMetrics(ActualWidth);
        var left = SidebarOcclusion.WindowX(Rows) + index % cols * itemWidth;
        return SidebarOcclusion.IsCovered(left, left + itemWidth - Gap, _bandLo, _bandHi);
    }

    public MusicGridView()
    {
        InitializeComponent();
        ScrollViewer.SetVerticalScrollBarVisibility(Rows, ScrollBarVisibility.Hidden);
        ScrollViewer.SetHorizontalScrollBarVisibility(Rows, ScrollBarVisibility.Disabled);
        Rows.ContainerContentChanging += OnContainerChanging;
        Rows.Loaded += (_, _) =>
        {
            if (Rows.ItemsPanelRoot is ItemsWrapGrid wrapGrid)
            {
                wrapGrid.ItemWidth = CardItemWidth;
            }
        };
        Rows.SizeChanged += (_, _) => UpdateMetrics(ActualWidth);
        SizeChanged += (_, e) => UpdateMetrics(e.NewSize.Width);
        Loaded += (_, _) => UpdateMetrics(ActualWidth);
    }

#if DEBUG
    static MusicGridView()
    {
        var (c1, w1) = ComputeMetrics(500);
        System.Diagnostics.Debug.Assert(c1 == 1 && w1 == 516);
        var (c2, w2) = ComputeMetrics(700);
        System.Diagnostics.Debug.Assert(c2 == 2 && w2 == 358);
        var (c3, w3) = ComputeMetrics(1050);
        System.Diagnostics.Debug.Assert(c3 == 3 && w3 == 355);
    }
#endif

    internal static (int Cols, double ItemWidth) ComputeMetrics(double width)
    {
        if (width <= 0)
        {
            return (1, MinColumnWidth);
        }
        // ItemWidth is the whole slot: card + trailing gap. The ListView is
        // Gap wider than the control, so the last column's gap falls outside.
        var available = Math.Max(1, width + Gap);
        var cols = Math.Max(1, (int)(available / (MinColumnWidth + Gap)));
        // ponytail: floor itemWidth to prevent sub-pixel rounding from wrapping last column prematurely.
        var itemWidth = Math.Max(1, Math.Floor(available / cols));
        return (cols, itemWidth);
    }

    private void UpdateMetrics(double width)
    {
        if (width <= 0)
        {
            return;
        }
        var (_, itemWidth) = ComputeMetrics(width);
        if (Math.Abs(itemWidth - CardItemWidth) < 0.5)
        {
            return;
        }
        CardItemWidth = itemWidth;
        if (Rows.ItemsPanelRoot is ItemsWrapGrid wrapGrid)
        {
            wrapGrid.ItemWidth = itemWidth;
        }
    }

    private static void OnLibraryStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (MusicGridView)d;
        if (e.OldValue is LibraryState oldLibrary)
        {
            oldLibrary.PropertyChanged -= self.OnLibraryChanged;
        }
        if (e.NewValue is LibraryState newLibrary)
        {
            newLibrary.PropertyChanged += self.OnLibraryChanged;
        }
        self.RefreshEmptyState();
    }

    private static void OnFilterStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (MusicGridView)d;
        if (e.OldValue is LibraryFilterState oldFilter)
        {
            oldFilter.PropertyChanged -= self.OnSelectionChanged;
        }
        if (e.NewValue is LibraryFilterState newFilter)
        {
            newFilter.PropertyChanged += self.OnSelectionChanged;
        }
        self.RefreshEmptyState();
    }

    private void OnSelectionChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName is nameof(LibraryFilterState.DisplayedTracks)
            or nameof(LibraryFilterState.Index)
            or nameof(LibraryFilterState.Detail))
        {
            RefreshEmptyState();
        }
        // Index alone changes every row's cache key. Detail/DisplayedTracks
        // fire alongside the same switch — recycling covers those containers,
        // and LoadInto no-ops repeats, so re-walking here only contends with
        // the backdrop frame loop for no gain.
        if (e.PropertyName == nameof(LibraryFilterState.Index))
        {
            RefreshVisibleCovers();
        }
    }

    private void OnLibraryChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
        => RefreshEmptyState();

    private void RefreshEmptyState()
    {
        var filter = Filter;
        var empty = filter is null || filter.DisplayedTracks.Count == 0;
        EmptyState.Visibility = empty ? Visibility.Visible : Visibility.Collapsed;
        Rows.Visibility = empty ? Visibility.Collapsed : Visibility.Visible;
        if (Library is { } library)
        {
            var libraryEmpty = library.Tracks.Count == 0;
            EmptyLabel.Text = libraryEmpty ? "No songs in library" : "No songs found";
            EmptyAction.Visibility = libraryEmpty ? Visibility.Visible : Visibility.Collapsed;
        }
    }

    private void RefreshVisibleCovers()
    {
        // Selection switches change every row's cache key — reload realized rows.
        foreach (var item in Rows.Items)
        {
            if (Rows.ContainerFromItem(item) is ListViewItem container)
            {
                PrepareContainer(container, (TrackItemViewModel)item);
            }
        }
    }

    private void OnContainerChanging(ListViewBase sender, ContainerContentChangingEventArgs args)
    {
        if (args.Item is TrackItemViewModel track && args.ItemContainer is ListViewItem container)
        {
            PrepareContainer(container, track);
            SidebarOcclusion.Apply(container, IsCovered(args.ItemIndex));
        }
    }

    private void PrepareContainer(ListViewItem container, TrackItemViewModel track)
    {
        var covers = Covers;
        var filter = Filter;
        if (covers is null || filter is null)
        {
            return;
        }
        var thumb = CoverImages.FindChild<Image>(container);
        var marker = CoverImages.FindChildren<FontIcon>(container)
            .FirstOrDefault(f => (f.Tag as string) == "now");
        if (thumb is null)
        {
            return;
        }

        // Inside an album every row shares the album cover key.
        _ = filter.Index is { } albumIndex
            ? covers.LoadAlbum(thumb, (ulong)albumIndex, Pixels, track)
            : covers.LoadTrack(thumb, track.Index, Pixels, track);

        if (marker is not null)
        {
            marker.Visibility = track.IsCurrent ? Visibility.Visible : Visibility.Collapsed;
        }
    }

    private void OnItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is TrackItemViewModel track)
        {
            Filter?.PlayTrack(track);
        }
    }

    private void OnOpenSettings(object sender, RoutedEventArgs e) => OpenSettingsRequested?.Invoke();
}
