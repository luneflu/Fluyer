using Fluyer.Core.State;
using Fluyer.Shared;
using Microsoft.UI;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Windows.Graphics;

namespace Fluyer;

/// <summary>
/// Library window shell: backdrop, carousel, header, grid, player bar, the
/// now-playing overlay and the toast.
/// </summary>
public sealed partial class MainWindow : Window
{
    public AppState State { get; }
    public CoverState Covers { get; }
    private readonly MediaTransportCoordinator _mediaTransport;

    public MainWindow(AppState state, CoverState covers)
    {
        State = state;
        Covers = covers;
        InitializeComponent();

        ExtendsContentIntoTitleBar = true;
        SetTitleBar(AppTitleBar);
        if (AppWindowTitleBar.IsCustomizationSupported() && AppWindow.TitleBar is not null)
        {
            AppWindow.TitleBar.PreferredHeightOption = TitleBarHeightOption.Tall;
        }
        AppWindow.SetIcon("Assets/AppIcon.ico");
        AppWindow.Resize(new SizeInt32(
            (int)Layout.Number("WindowInitialWidth"), (int)Layout.Number("WindowInitialHeight")));

        _mediaTransport = new MediaTransportCoordinator(this, state.Playback, covers);
        Closed += (_, _) => _mediaTransport.Dispose();

        State.Playback.PropertyChanged += OnPlaybackChanged;
        State.Library.PropertyChanged += OnLibraryChanged;
        State.Selection.PropertyChanged += OnSelectionChanged;
        Settings.State = state;
        Settings.AddFolderRequested += () => _ = PickAndScanAsync();
        MusicGrid.OpenSettingsRequested += ShowSettings;
        // Fires after the value changes on every path (button, shortcut, light dismiss, Esc).
        QueuePane.RegisterPropertyChangedCallback(Microsoft.UI.Xaml.Controls.SplitView.IsPaneOpenProperty, (_, _) =>
        {
            RefreshOcclusion();
            RefreshMenu();
        });

        State.Library.Tracks.CollectionChanged += (_, _) => RefreshMenu();
        RefreshOverlays();
        RefreshScan();
        RefreshMenu();
        RefreshMode();
    }

    private void OnPlaybackChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(PlaybackState.ShowPlayView))
        {
            RefreshOverlays();
        }
        if (e.PropertyName is nameof(PlaybackState.Bar))
        {
            RefreshMenu();
        }
    }

    private void OnLibraryChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(LibraryState.ScanStatus))
        {
            RefreshScan();
        }
    }

    private void OnSelectionChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName is nameof(LibraryFilterState.Index) or nameof(LibraryFilterState.IsActive)
            or nameof(LibraryFilterState.Mode))
        {
            RefreshMode();
        }
    }

    /// <summary>Albums mode: full-height album grid. Songs mode: carousel, album header, song grid.</summary>
    private void RefreshMode()
    {
        var albums = State.Selection.Mode == LibraryMode.Albums;
        AlbumGrid.Visibility = albums ? Visibility.Visible : Visibility.Collapsed;
        MusicGrid.Visibility = albums ? Visibility.Collapsed : Visibility.Visible;
        AlbumHeader.Visibility = !albums && State.Selection.IsActive ? Visibility.Visible : Visibility.Collapsed;
        SongsMode.IsChecked = !albums;
        AlbumsMode.IsChecked = albums;
    }

    private void OnShowSongs(object sender, RoutedEventArgs e) => State.Selection.Mode = LibraryMode.Tracks;
    private void OnShowAlbums(object sender, RoutedEventArgs e) => State.Selection.Mode = LibraryMode.Albums;

    private void RefreshOverlays()
    {
        var showing = State.Playback.ShowPlayView;
        if (showing)
        {
            QueuePane.IsPaneOpen = false;
        }
        NowPlaying.Visibility = showing ? Visibility.Visible : Visibility.Collapsed;
        LibraryRoot.Visibility = showing ? Visibility.Collapsed : Visibility.Visible;
    }

    private void RefreshScan()
    {
        var status = State.Library.ScanStatus;
        ScanStatus.Visibility = status.IsScanning ? Visibility.Visible : Visibility.Collapsed;
        ScanLabel.Text = status.StatusLabel;
    }

    private void OnSearchChanged(Microsoft.UI.Xaml.Controls.AutoSuggestBox sender,
        Microsoft.UI.Xaml.Controls.AutoSuggestBoxTextChangedEventArgs args)
        => State.Selection.Query = sender.Text;

    private async Task PickAndScanAsync()
    {
        var paths = await FolderPicker.PickMusicFoldersAsync(this);
        State.ScanFolders(paths);
        Settings.Refresh();
    }

    // MARK: - Menu bar (same commands as the macOS menu bar)

    private void RefreshMenu()
    {
        PlayAllItem.IsEnabled = State.Library.Tracks.Count > 0;
        PlayPauseItem.Text = State.Playback.Bar.IsPlaying ? "Pause" : "Play";
        QueueItem.Text = QueuePane.IsPaneOpen ? "Hide Queue" : "Show Queue";
        QueueToggle.IsChecked = QueuePane.IsPaneOpen;
        AutomationProperties.SetName(QueueToggle, QueueItem.Text);
    }

    private void OnOpenFolder(object sender, RoutedEventArgs e) => _ = PickAndScanAsync();
    private void OnOpenSettings(object sender, RoutedEventArgs e) => ShowSettings();
    private void OnPlayAll(object sender, RoutedEventArgs e) => State.PlayAll();
    private void OnTogglePlay(object sender, RoutedEventArgs e) => State.Playback.TogglePlay();
    private void OnNext(object sender, RoutedEventArgs e) => State.Playback.Next();
    private void OnPrevious(object sender, RoutedEventArgs e) => State.Playback.Previous();
    private void OnShowPlayScreen(object sender, RoutedEventArgs e) => State.Playback.ShowPlayView = true;
    private void OnToggleQueue(object sender, RoutedEventArgs e) => ToggleQueue();

    private void ShowSettings()
    {
        Settings.Refresh();
        SettingsDialog.XamlRoot = Content.XamlRoot;
        _ = SettingsDialog.ShowAsync();
    }

    // MARK: - Queue sidebar (button / Ctrl+L; SplitView overlay closes on outside click + Esc)

    private void ToggleQueue()
    {
        QueuePane.IsPaneOpen = !QueuePane.IsPaneOpen;
    }

    // The queue pane is two carousel covers + the gap between (Sidebar.svelte: itemWidth * 2).
    // Carousel sits inside WindowPageGutter, so measure the same width.
    private void OnPaneHostSizeChanged(object sender, SizeChangedEventArgs e)
    {
        var gutter = Layout.Edges("WindowPageGutter");
        var carouselWidth = e.NewSize.Width - gutter.Left - gutter.Right;
        var length = Math.Floor(Screens.AlbumCarouselView.ItemWidth(carouselWidth) * 2
            - Screens.AlbumCarouselView.Gap);
        QueuePane.OpenPaneLength = length;
        RefreshOcclusion();
    }

    private void OnQueuePaneOpening(Microsoft.UI.Xaml.Controls.SplitView sender, object args)
        => State.Queue.IsOpen = true;

    private void OnQueuePaneClosed(Microsoft.UI.Xaml.Controls.SplitView sender, object args)
        => State.Queue.IsOpen = false;

    // Sidebar.svelte hides the items behind an open pane (useAlbumList/useMusicList).
    private void RefreshOcclusion()
    {
        var lo = double.NegativeInfinity;
        var hi = QueuePane.IsPaneOpen ? QueuePane.ActualWidth - QueuePane.OpenPaneLength : double.PositiveInfinity;
        Carousel.SetVisibleBand(lo, hi);
        MusicGrid.SetVisibleBand(lo, hi);
    }
}
