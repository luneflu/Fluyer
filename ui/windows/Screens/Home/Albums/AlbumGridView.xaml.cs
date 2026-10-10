using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Shared;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Layout = Fluyer.Shared.Layout;

namespace Fluyer.Screens;

/// <summary>
/// Full-height album grid shown in Albums mode. Click opens the album (and returns
/// to Songs mode); right-click plays / queues / shuffles it.
/// </summary>
public sealed partial class AlbumGridView : UserControl
{
    public static readonly DependencyProperty LibraryProperty =
        DependencyProperty.Register(nameof(Library), typeof(LibraryState), typeof(AlbumGridView),
            new PropertyMetadata(null, (d, e) => ((AlbumGridView)d).RefreshEmpty()));

    public static readonly DependencyProperty FilterProperty =
        DependencyProperty.Register(nameof(Filter), typeof(LibraryFilterState), typeof(AlbumGridView),
            new PropertyMetadata(null, OnFilterStateChanged));

    public static readonly DependencyProperty CoversProperty =
        DependencyProperty.Register(nameof(Covers), typeof(CoverState), typeof(AlbumGridView),
            new PropertyMetadata(null));

    public static readonly DependencyProperty CardCoverSizeProperty =
        DependencyProperty.Register(nameof(CardCoverSize), typeof(double), typeof(AlbumGridView),
            new PropertyMetadata(148.0));

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

    public AlbumGridView()
    {
        InitializeComponent();
        Cards.ContainerContentChanging += OnContainerChanging;
        SizeChanged += (_, e) => UpdateMetrics(e.NewSize.Width);
    }

    private static void OnFilterStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (AlbumGridView)d;
        if (e.OldValue is LibraryFilterState oldFilter)
        {
            oldFilter.PropertyChanged -= self.OnFilterChanged;
        }
        if (e.NewValue is LibraryFilterState newFilter)
        {
            newFilter.PropertyChanged += self.OnFilterChanged;
        }
        self.RefreshEmpty();
    }

    private void OnFilterChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(LibraryFilterState.DisplayedAlbums))
        {
            RefreshEmpty();
        }
    }

    private void RefreshEmpty()
    {
        var empty = (Filter?.DisplayedAlbums.Count ?? 0) == 0;
        EmptyLabel.Visibility = empty ? Visibility.Visible : Visibility.Collapsed;
        EmptyLabel.Text = (Library?.Albums.Count ?? 0) == 0 ? "No albums in library" : "No albums found";
    }

    // Same width rule as the carousel, minus the grid's own side padding.
    private void UpdateMetrics(double width)
    {
        var padding = Layout.Edges("AlbumsGridPadding");
        var inner = width - padding.Left - padding.Right;
        if (inner <= 0)
        {
            return;
        }
        var cover = Math.Max(16, Math.Floor(AlbumCarouselView.ItemWidth(inner)
            - AlbumCarouselView.Gap - AlbumCarouselView.ItemPaddingX));
        if (Math.Abs(cover - CardCoverSize) >= 0.5)
        {
            CardCoverSize = cover;
        }
    }

    private void OnContainerChanging(ListViewBase sender, ContainerContentChangingEventArgs args)
    {
        if (args.Item is AlbumCardViewModel album
            && Covers is { } covers
            && CoverImages.FindChild<Image>(args.ItemContainer) is { } cover)
        {
            // Fixed 2x like the carousel, quantized to 32px so both share cache keys.
            var pixels = Math.Max(16, ((int)(CardCoverSize * 2) / 32) * 32);
            _ = covers.LoadAlbum(cover, album.Index, pixels, album);
        }
    }

    private void OnItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is AlbumCardViewModel album)
        {
            Filter?.Select((int)album.Index);
        }
    }

    private static int AlbumIndex(object sender) => (int)(ulong)((FrameworkElement)sender).Tag;

    private void OnPlayAlbum(object sender, RoutedEventArgs e) => Filter?.PlayAlbum(AlbumIndex(sender));
    private void OnQueueAlbum(object sender, RoutedEventArgs e) => Filter?.QueueAlbum(AlbumIndex(sender));
    private void OnShuffleAlbum(object sender, RoutedEventArgs e) => Filter?.ShuffleAlbum(AlbumIndex(sender));
}
