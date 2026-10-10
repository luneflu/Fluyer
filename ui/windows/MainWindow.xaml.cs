using Fluyer.Core.State;
using Fluyer.Shared;
using Microsoft.UI;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
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
        MenuPaneContent.SettingsRequested += ShowSettings;
        MenuPaneContent.PlayAllRequested += State.PlayAll;
        MenuPaneContent.CloseRequested += () => MenuPane.IsPaneOpen = false;
        PlayerBar.QueueRequested += ToggleQueue;
        AddShortcut(Windows.System.VirtualKey.Q, Windows.System.VirtualKeyModifiers.Control, ToggleQueue);
        AddShortcut(Windows.System.VirtualKey.M, Windows.System.VirtualKeyModifiers.Control, ToggleMenu);
        // Fires after the value changes on every path (button, shortcut, light dismiss, Esc).
        MenuPane.RegisterPropertyChangedCallback(Microsoft.UI.Xaml.Controls.SplitView.IsPaneOpenProperty, (_, _) => RefreshOcclusion());
        QueuePane.RegisterPropertyChangedCallback(Microsoft.UI.Xaml.Controls.SplitView.IsPaneOpenProperty, (_, _) => RefreshOcclusion());

        RefreshOverlays();
        RefreshScan();
    }

    private void OnPlaybackChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(PlaybackState.ShowPlayView))
        {
            RefreshOverlays();
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
        if (e.PropertyName is nameof(LibraryFilterState.Index) or nameof(LibraryFilterState.IsActive))
        {
            AlbumHeader.Visibility = State.Selection.IsActive
                ? Visibility.Visible : Visibility.Collapsed;
        }
    }

    private void RefreshOverlays()
    {
        var showing = State.Playback.ShowPlayView;
        if (showing)
        {
            MenuPane.IsPaneOpen = false;
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

    private void OnOpenSettings(object sender, RoutedEventArgs e) => ShowSettings();

    private void ShowSettings()
    {
        Settings.Refresh();
        SettingsDialog.XamlRoot = Content.XamlRoot;
        _ = SettingsDialog.ShowAsync();
    }

    // MARK: - Sidebars (button / shortcut opened; SplitView overlay closes on outside click + Esc)

    private void AddShortcut(Windows.System.VirtualKey key, Windows.System.VirtualKeyModifiers modifiers, Action action)
    {
        var accelerator = new Microsoft.UI.Xaml.Input.KeyboardAccelerator { Key = key, Modifiers = modifiers };
        accelerator.Invoked += (_, args) =>
        {
            args.Handled = true;
            action();
        };
        LibraryRoot.KeyboardAccelerators.Add(accelerator);
    }

    private void ToggleMenu()
    {
        QueuePane.IsPaneOpen = false;
        MenuPane.IsPaneOpen = !MenuPane.IsPaneOpen;
    }

    private void ToggleQueue()
    {
        MenuPane.IsPaneOpen = false;
        QueuePane.IsPaneOpen = !QueuePane.IsPaneOpen;
    }

    private void OnMenuToggle(Microsoft.UI.Xaml.Controls.TitleBar sender, object args) => ToggleMenu();

    // Sidebars are two carousel covers + the gap between (Sidebar.svelte: itemWidth * 2).
    // Carousel sits inside WindowPageGutter, so measure the same width.
    private void OnPaneHostSizeChanged(object sender, SizeChangedEventArgs e)
    {
        var gutter = Layout.Edges("WindowPageGutter");
        var carouselWidth = e.NewSize.Width - gutter.Left - gutter.Right;
        var length = Math.Floor(Screens.AlbumCarouselView.ItemWidth(carouselWidth) * 2
            - Screens.AlbumCarouselView.Gap);
        MenuPane.OpenPaneLength = length;
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
        var lo = MenuPane.IsPaneOpen ? MenuPane.OpenPaneLength : double.NegativeInfinity;
        var hi = QueuePane.IsPaneOpen ? MenuPane.ActualWidth - QueuePane.OpenPaneLength : double.PositiveInfinity;
        Carousel.SetVisibleBand(lo, hi);
        MusicGrid.SetVisibleBand(lo, hi);
    }
}
