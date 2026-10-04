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

    public AppState? State
    {
        get => (AppState?)GetValue(StateProperty);
        set => SetValue(StateProperty, value);
    }

    /// <summary>Raised when the empty-state button needs a folder pick (handled by the window).</summary>
    public event Action? OpenFolderRequested;

    // x:Bind static function binding for the artist • album line.
    public static string ArtistAlbumLine(string artist, string album) => $"{artist} • {album}";

    private const int Pixels = 88;

    public MusicGridView()
    {
        InitializeComponent();
        Rows.ContainerContentChanging += OnContainerChanging;
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
        Func<byte[]?> load;
        if (state!.Selection.Index is { } albumIndex)
        {
            var i = (ulong)albumIndex;
            key = ThumbnailKey.Album(i, Pixels);
            load = () => engine.GetAlbumThumbnail(i, Pixels);
        }
        else
        {
            var i = track.Index;
            key = ThumbnailKey.Track(i, Pixels);
            load = () => engine.GetTrackThumbnail(i, Pixels);
        }
        _ = CoverImages.LoadInto(thumb, key, track, load);

        if (marker is not null)
        {
            marker.Visibility = track.IsCurrent ? Visibility.Visible : Visibility.Collapsed;
        }
    }

    private void OnItemClick(object sender, ItemClickEventArgs e)
    {
        var state = State;
        if (state is null || e.ClickedItem is not TrackItemViewModel track)
        {
            return;
        }
        var row = state.Selection.DisplayedTracks.ToList().IndexOf(track);
        state.Selection.PlayTrackAtRow(row);
    }

    private void OnOpenFolder(object sender, RoutedEventArgs e) => OpenFolderRequested?.Invoke();
}
