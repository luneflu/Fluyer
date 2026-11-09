using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Shared;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
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
        if (e.PropertyName is nameof(LibraryFilterState.Index) or nameof(LibraryFilterState.DisplayedAlbums)
            or nameof(LibraryFilterState.Mode))
        {
            RefreshChrome();
        }
    }

    private bool _listHovered;

    private void OnListPointerEntered(object sender, PointerRoutedEventArgs e)
    {
        _listHovered = true;
        RefreshChrome();
    }

    private void OnListPointerExited(object sender, PointerRoutedEventArgs e)
    {
        _listHovered = false;
        RefreshChrome();
    }

    private void RefreshChrome()
    {
        foreach (var item in Cards.Items)
        {
            if (Cards.ContainerFromItem(item) is GridViewItem container && item is AlbumCardViewModel album)
            {
                ApplyChrome(container, album);
            }
        }
    }

    // AlbumCard.swift: 40% opacity for the rest while an album is open, lifted while the list is hovered.
    private void ApplyChrome(GridViewItem container, AlbumCardViewModel album)
    {
        if (container.ContentTemplateRoot is not FrameworkElement card)
        {
            return;
        }
        var selected = Filter?.Index == (int)album.Index;
        var dimmed = Filter?.IsActive == true && !selected && !_listHovered;
        AlbumCardChrome.Apply(card, selected, dimmed);
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
        if (args.Item is AlbumCardViewModel album)
        {
            if (args.ItemContainer is GridViewItem container)
            {
                ApplyChrome(container, album);
            }
            if (Covers is { } covers
                && CoverImages.FindChild<Image>(args.ItemContainer) is { } cover)
            {
                // Fixed 2x like the carousel, quantized to 32px so both share cache keys.
                var pixels = Math.Max(16, ((int)(CardCoverSize * 2) / 32) * 32);
                _ = LoadCoverAsync(covers, cover, album, pixels);
            }
        }
    }

    // Placeholder stays until art actually paints (transparent PNGs would show it through).
    private static async Task LoadCoverAsync(CoverState covers, Image cover, AlbumCardViewModel album, int pixels)
    {
        AlbumCardChrome.ShowPlaceholder(cover, cover.Source is null);
        await covers.LoadAlbum(cover, album.Index, pixels, album);
        AlbumCardChrome.ShowPlaceholder(cover, cover.Source is null);
    }

    private void OnCardPointerEntered(object sender, PointerRoutedEventArgs e) => SetHover(sender, true);
    private void OnCardPointerExited(object sender, PointerRoutedEventArgs e) => SetHover(sender, false);

    private void SetHover(object sender, bool hovering)
    {
        if (sender is FrameworkElement card)
        {
            var selected = card.DataContext is AlbumCardViewModel album
                && Filter?.Index == (int)album.Index;
            AlbumCardChrome.Hover(card, hovering, selected);
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
