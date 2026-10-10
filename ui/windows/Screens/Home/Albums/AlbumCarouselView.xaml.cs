using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Core.Support;
using Fluyer.Components;
using Fluyer.Shared;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Layout = Fluyer.Shared.Layout;
using Windows.Graphics.Display;

namespace Fluyer.Screens;

/// <summary>
/// Horizontal strip of album covers; tapping drills the grid into that album.
/// </summary>
public sealed partial class AlbumCarouselView : UserControl
{
    public static readonly DependencyProperty LibraryProperty =
        DependencyProperty.Register(nameof(Library), typeof(LibraryState), typeof(AlbumCarouselView),
            new PropertyMetadata(null, OnLibraryStateChanged));

    public static readonly DependencyProperty FilterProperty =
        DependencyProperty.Register(nameof(Filter), typeof(LibraryFilterState), typeof(AlbumCarouselView),
            new PropertyMetadata(null, OnFilterStateChanged));

    public static readonly DependencyProperty CoversProperty =
        DependencyProperty.Register(nameof(Covers), typeof(CoverState), typeof(AlbumCarouselView),
            new PropertyMetadata(null));

    public static readonly DependencyProperty CardCoverSizeProperty =
        DependencyProperty.Register(nameof(CardCoverSize), typeof(double), typeof(AlbumCarouselView),
            new PropertyMetadata(148.0));

    public static readonly DependencyProperty CarouselHeightProperty =
        DependencyProperty.Register(nameof(CarouselHeight), typeof(double), typeof(AlbumCarouselView),
            new PropertyMetadata(200.0));

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

    public double CardCoverSize
    {
        get => (double)GetValue(CardCoverSizeProperty);
        private set => SetValue(CardCoverSizeProperty, value);
    }

    public double CarouselHeight
    {
        get => (double)GetValue(CarouselHeightProperty);
        private set => SetValue(CarouselHeightProperty, value);
    }

    // (minWidth, minDpr, widthRatio) — HiDPI rows first, mirroring macOS.
    private static readonly (double MinWidth, double MinDpr, double Ratio)[] Rules =
    [
        (1536, 1.01, 0.142857), (1280, 1.01, 0.16667), (1024, 1.01, 0.2),
        (768, 1.01, 0.25), (640, 1.01, 0.33334),
        (1536, 0, 0.125), (1440, 0, 0.142857), (1280, 0, 0.16667),
        (1024, 0, 0.2), (768, 0, 0.25), (640, 0, 0.33334),
    ];

    // Read from Shared/Theme/Layout.xaml (Albums*) so the slot math matches the XAML.
    private static double LabelHeight => Layout.Number("AlbumsLabelHeight");
    private static double Spacing => Layout.Number("AlbumsCoverToLabels");
    public static double Gap => Layout.Edges("AlbumsItemMargin").Right;
    // Item padding insets the hover backplate (~4,2) so content has a 4px cushion all around.
    public static double ItemPaddingX => Layout.Edges("AlbumsItemPadding") is var p ? p.Left + p.Right : 0;
    public static double ItemPaddingY => Layout.Edges("AlbumsItemPadding") is var p ? p.Top + p.Bottom : 0;

    // Window-X band left visible by open sidebars (SidebarOcclusion).
    private double _bandLo = double.NegativeInfinity;
    private double _bandHi = double.PositiveInfinity;
    private ScrollViewer? _scroller;

    public AlbumCarouselView()
    {
        InitializeComponent();
        ScrollViewer.SetHorizontalScrollBarVisibility(Strip, ScrollBarVisibility.Hidden);
        Strip.ContainerContentChanging += OnContainerChanging;
        SizeChanged += (_, e) => UpdateMetrics(e.NewSize.Width);
        Loaded += (_, _) =>
        {
            UpdateMetrics(ActualWidth);
            _scroller = CoverImages.FindChild<ScrollViewer>(Strip);
            if (_scroller is not null)
            {
                _scroller.ViewChanged += (_, _) => RefreshOcclusion();
            }
        };
    }

    /// <summary>Fades cards outside window-X [lo, hi] (under an open sidebar).</summary>
    public void SetVisibleBand(double lo, double hi)
    {
        _bandLo = lo;
        _bandHi = hi;
        RefreshOcclusion();
    }

    // Positions are computed, not measured: recycled containers aren't arranged yet
    // in ContainerContentChanging. Slot = cover + item padding + gap.
    private bool IsCovered(int index)
    {
        if (double.IsNegativeInfinity(_bandLo) && double.IsPositiveInfinity(_bandHi))
        {
            return false;
        }
        var slot = CardCoverSize + ItemPaddingX + Gap;
        var left = SidebarOcclusion.WindowX(Strip) + index * slot - (_scroller?.HorizontalOffset ?? 0);
        return SidebarOcclusion.IsCovered(left, left + slot - Gap, _bandLo, _bandHi);
    }

    private void RefreshOcclusion() => SidebarOcclusion.ApplyAll(Strip, IsCovered);

    private static void OnLibraryStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (AlbumCarouselView)d;
        if (e.OldValue is LibraryState oldLibrary)
        {
            oldLibrary.PropertyChanged -= self.OnLibraryChanged;
        }
        if (e.NewValue is LibraryState newLibrary)
        {
            newLibrary.PropertyChanged += self.OnLibraryChanged;
        }
        self.RefreshVisibility();
    }

    private static void OnFilterStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (AlbumCarouselView)d;
        if (e.OldValue is LibraryFilterState oldFilter)
        {
            oldFilter.PropertyChanged -= self.OnSelectionChanged;
        }
        if (e.NewValue is LibraryFilterState newFilter)
        {
            newFilter.PropertyChanged += self.OnSelectionChanged;
        }
        self.RefreshVisibility();
    }

    private void OnLibraryChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
        => RefreshVisibility();

    private void OnSelectionChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        // Emphasis only depends on Index; Detail/DisplayedTracks fire
        // alongside the same switch and re-walking here contends with the
        // backdrop frame loop for no gain (recycle re-applies via PrepareContainer).
        if (string.IsNullOrEmpty(e.PropertyName) || e.PropertyName == nameof(LibraryFilterState.Index))
        {
            RefreshSelectionEmphasis();
        }
    }

    private void RefreshVisibility()
        => Visibility = Library is { } library && library.Albums.Count > 0
            ? Visibility.Visible : Visibility.Collapsed;

    private static double Dpr()
    {
        // Layout-rule input only (mirrors NSScreen.backingScaleFactor).
        // Pixel requests intentionally ignore this and use fixed 2x like
        // Swift — see PixelSize.
        try
        {
            return DisplayInformation.GetForCurrentView().RawPixelsPerViewPixel;
        }
        catch
        {
            return 1.0;
        }
    }

    /// <summary>
    /// Slot width (cover + one gap) for a carousel of <paramref name="width"/>.
    /// Sidebars are two slots wide, like Svelte <c>sidebarStore.width = itemWidth * 2</c>.
    /// </summary>
    public static double ItemWidth(double width)
    {
        var dpr = Dpr();
        // Ratios split width + one trailing gap into whole slots, so n covers
        // and n-1 gaps fill the width exactly.
        var available = Math.Max(1, width + Gap);
        foreach (var (minWidth, minDpr, ratio) in Rules)
        {
            if (width >= minWidth && dpr >= minDpr)
            {
                return ratio * available;
            }
        }
        return 0.5 * available;
    }

    private void UpdateMetrics(double width)
    {
        if (width <= 0)
        {
            return;
        }
        var cover = Math.Max(16, Math.Floor(ItemWidth(width) - Gap - ItemPaddingX));
        // DP sets re-measure every card and resize the Strip row, which
        // resizes the backdrop and reallocates its D3D targets — so ignore
        // sub-pixel drift and never re-walk covers here (recycle already
        // fires OnContainerChanging for realized rows).
        if (Math.Abs(cover - CardCoverSize) < 0.5)
        {
            return;
        }
        CardCoverSize = cover;
        CarouselHeight = cover + LabelHeight + Spacing + ItemPaddingY;
    }

    private int PixelSize()
        // Fixed 2x like Swift (coverSize * 2), quantized to 32px so resizes
        // reuse cache entries. Prior code used the live monitor scale, which
        // minted small keys on 1x displays (e.g. 148px source shown at ~300px
        // wide) and every card upscaled soft.
        => Math.Max(16, ((int)(CardCoverSize * 2) / 32) * 32);

    private void RefreshSelectionEmphasis()
    {
        if (Filter is not { } filter)
        {
            return;
        }
        foreach (var item in Strip.Items)
        {
            if (Strip.ContainerFromItem(item) is ListViewItem container
                && item is AlbumCardViewModel album)
            {
                var label = FindTitle(container);
                if (label is not null)
                {
                    var selected = filter.Index == (int)album.Index;
                    label.FontWeight = selected
                        ? Microsoft.UI.Text.FontWeights.SemiBold
                        : Microsoft.UI.Text.FontWeights.Medium;
                }
            }
        }
    }

    private static TextBlock? FindTitle(DependencyObject container)
        => CoverImages.FindChildren<TextBlock>(container)
            .FirstOrDefault(t => (t.Tag as string) == "albumtitle");

    private void OnContainerChanging(ListViewBase sender, ContainerContentChangingEventArgs args)
    {
        if (args.Item is AlbumCardViewModel album && args.ItemContainer is ListViewItem container)
        {
            PrepareContainer(container, album);
            SidebarOcclusion.Apply(container, IsCovered(args.ItemIndex));
        }
    }

    private void PrepareContainer(ListViewItem container, AlbumCardViewModel album)
    {
        var covers = Covers;
        if (covers is null)
        {
            return;
        }
        // Requested at display scale — also what keeps this key distinct from
        // the 88px album thumbnail the track rows ask for.
        var pixels = PixelSize();
        var cover = CoverImages.FindChild<Image>(container);
        if (cover is not null)
        {
            _ = covers.LoadAlbum(cover, album.Index, pixels, album);
        }
        var label = FindTitle(container);
        if (label is not null && Filter is { } filter)
        {
            label.FontWeight = filter.Index == (int)album.Index
                ? Microsoft.UI.Text.FontWeights.SemiBold
                : Microsoft.UI.Text.FontWeights.Medium;
        }
    }

    private void OnItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is AlbumCardViewModel album)
        {
            Filter?.Select((int)album.Index);
        }
    }
}
