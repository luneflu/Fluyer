using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Core.Support;
using Fluyer.Support;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Windows.Graphics.Display;

namespace Fluyer.Views;

/// <summary>
/// Horizontal strip of album covers; tapping drills the grid into that album.
/// </summary>
public sealed partial class AlbumCarouselView : UserControl
{
    public static readonly DependencyProperty StateProperty =
        DependencyProperty.Register(nameof(State), typeof(AppState), typeof(AlbumCarouselView),
            new PropertyMetadata(null, OnStateChanged));

    public static readonly DependencyProperty CardCoverSizeProperty =
        DependencyProperty.Register(nameof(CardCoverSize), typeof(double), typeof(AlbumCarouselView),
            new PropertyMetadata(148.0));

    public static readonly DependencyProperty CarouselHeightProperty =
        DependencyProperty.Register(nameof(CarouselHeight), typeof(double), typeof(AlbumCarouselView),
            new PropertyMetadata(200.0));

    public AppState? State
    {
        get => (AppState?)GetValue(StateProperty);
        set => SetValue(StateProperty, value);
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

    private const double LabelHeight = 52;
    private const double Spacing = 5;
    public const double Gap = 16;
    // Padding must match ListViewItem.Padding in XAML (8,6,8,6):
    // Insets hover backplate (~4,2) so content has 4px cushion all around.
    public const double ItemPaddingX = 16; // 8 left + 8 right
    public const double ItemPaddingY = 12; // 6 top + 6 bottom

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

    private static void OnStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (AlbumCarouselView)d;
        if (e.OldValue is AppState oldState)
        {
            oldState.Library.PropertyChanged -= self.OnLibraryChanged;
            oldState.Selection.PropertyChanged -= self.OnSelectionChanged;
        }
        if (e.NewValue is AppState newState)
        {
            newState.Library.PropertyChanged += self.OnLibraryChanged;
            newState.Selection.PropertyChanged += self.OnSelectionChanged;
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
        if (string.IsNullOrEmpty(e.PropertyName) || e.PropertyName == nameof(AlbumSelection.Index))
        {
            RefreshSelectionEmphasis();
        }
    }

    private void RefreshVisibility()
        => Visibility = State is not null && State.Library.Albums.Count > 0
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
        if (State is null)
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
                    var selected = State.Selection.Index == (int)album.Index;
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
        var engine = State?.Engine;
        if (engine is null)
        {
            return;
        }
        // Requested at display scale — also what keeps this key distinct from
        // the 88px album thumbnail the track rows ask for.
        var pixels = PixelSize();
        var key = ThumbnailKey.Album(album.Index, pixels);
        var cover = CoverImages.FindChild<Image>(container);
        if (cover is not null)
        {
            var index = album.Index;
            _ = CoverImages.LoadInto(cover, key, album, () => engine.GetAlbumThumbnailAsync(index, (uint)pixels));
        }
        var label = FindTitle(container);
        if (label is not null && State is not null)
        {
            label.FontWeight = State.Selection.Index == (int)album.Index
                ? Microsoft.UI.Text.FontWeights.SemiBold
                : Microsoft.UI.Text.FontWeights.Medium;
        }
    }

    private void OnItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is AlbumCardViewModel album)
        {
            State?.Selection.Select((int)album.Index);
        }
    }
}
