using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Core.Support;
using Fluyer.Support;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Views;

/// <summary>
/// Track rows. Thumbnails load per realized container (the recycle-aware
/// equivalent of SwiftUI's <c>.task(id:)</c>); inside an album every row shares
/// the album cover key, otherwise each row uses its own track cover.
/// </summary>
public sealed partial class MusicGridView : UserControl
{
    public static readonly DependencyProperty StateProperty =
        DependencyProperty.Register(nameof(State), typeof(AppState), typeof(MusicGridView),
            new PropertyMetadata(null, OnStateChanged));

    public static readonly DependencyProperty CardItemWidthProperty =
        DependencyProperty.Register(nameof(CardItemWidth), typeof(double), typeof(MusicGridView),
            new PropertyMetadata(280.0));

    public AppState? State
    {
        get => (AppState?)GetValue(StateProperty);
        set => SetValue(StateProperty, value);
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

    private const int Pixels = 88;

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
        System.Diagnostics.Debug.Assert(c1 == 1 && w1 == 476);
        var (c2, w2) = ComputeMetrics(700);
        System.Diagnostics.Debug.Assert(c2 == 2 && w2 == 338);
        var (c3, w3) = ComputeMetrics(1050);
        System.Diagnostics.Debug.Assert(c3 == 3 && w3 == 342);
    }
#endif

    internal static (int Cols, double ItemWidth) ComputeMetrics(double width)
    {
        if (width <= 0)
        {
            return (1, 280);
        }
        // ItemWidth is the whole slot (hover backplate insets itself inside it),
        // so only the ListView padding (12 per side) comes off the top.
        const double listPadding = 24;
        const double minColWidth = 280;
        var available = Math.Max(1, width - listPadding);
        var cols = Math.Max(1, (int)(available / minColWidth));
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

    private static void OnStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (MusicGridView)d;
        if (e.OldValue is AppState oldState)
        {
            oldState.Selection.PropertyChanged -= self.OnSelectionChanged;
            oldState.Library.PropertyChanged -= self.OnLibraryChanged;
        }
        if (e.NewValue is AppState newState)
        {
            newState.Selection.PropertyChanged += self.OnSelectionChanged;
            newState.Library.PropertyChanged += self.OnLibraryChanged;
        }
        self.RefreshEmptyState();
    }

    private void OnSelectionChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName is nameof(AlbumSelection.DisplayedTracks)
            or nameof(AlbumSelection.Index)
            or nameof(AlbumSelection.Detail))
        {
            RefreshEmptyState();
        }
        // Index alone changes every row's cache key. Detail/DisplayedTracks
        // fire alongside the same switch — recycling covers those containers,
        // and LoadInto no-ops repeats, so re-walking here only contends with
        // the backdrop frame loop for no gain.
        if (e.PropertyName == nameof(AlbumSelection.Index))
        {
            RefreshVisibleCovers();
        }
    }

    private void OnLibraryChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
        => RefreshEmptyState();

    private void RefreshEmptyState()
    {
        var state = State;
        var empty = state is null || state.Selection.DisplayedTracks.Count == 0;
        EmptyState.Visibility = empty ? Visibility.Visible : Visibility.Collapsed;
        Rows.Visibility = empty ? Visibility.Collapsed : Visibility.Visible;
        if (state is not null)
        {
            var libraryEmpty = state.Library.Tracks.Count == 0;
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
        }
    }

    private void PrepareContainer(ListViewItem container, TrackItemViewModel track)
    {
        var state = State;
        var engine = state?.Engine;
        if (engine is null)
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

        string key;
        Func<Task<byte[]?>> load;
        if (state!.Selection.Index is { } albumIndex)
        {
            var i = (ulong)albumIndex;
            key = ThumbnailKey.Album(i, Pixels);
            load = () => engine.GetAlbumThumbnailAsync(i, Pixels);
        }
        else
        {
            var i = track.Index;
            key = ThumbnailKey.Track(i, Pixels);
            load = () => engine.GetTrackThumbnailAsync(i, Pixels);
        }
        _ = CoverImages.LoadInto(thumb, key, track, load);

        if (marker is not null)
        {
            marker.Visibility = track.IsCurrent ? Visibility.Visible : Visibility.Collapsed;
        }
    }

    private void OnItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is TrackItemViewModel track)
        {
            State?.Selection.PlayTrack(track);
        }
    }

    private void OnOpenSettings(object sender, RoutedEventArgs e) => OpenSettingsRequested?.Invoke();
}
