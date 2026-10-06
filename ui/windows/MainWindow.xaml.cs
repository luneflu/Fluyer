using Fluyer.Core.State;
using Fluyer.Support;
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
    private readonly MediaTransportCoordinator _mediaTransport;

    public MainWindow(AppState state)
    {
        State = state;
        InitializeComponent();

        ExtendsContentIntoTitleBar = true;
        SetTitleBar(AppTitleBar);
        AppWindow.SetIcon("Assets/AppIcon.ico");
        AppWindow.Resize(new SizeInt32(1050, 720));

        _mediaTransport = new MediaTransportCoordinator(this, state);
        Closed += (_, _) => _mediaTransport.Dispose();

        State.Playback.PropertyChanged += OnPlaybackChanged;
        State.Library.PropertyChanged += OnLibraryChanged;
        State.Toast.PropertyChanged += OnToastChanged;
        State.Selection.PropertyChanged += OnSelectionChanged;
        MusicGrid.OpenSettingsRequested += ShowSettings;
        PlayerBar.QueueRequested += ToggleQueue;
        AddShortcut(Windows.System.VirtualKey.Q, Windows.System.VirtualKeyModifiers.Control, ToggleQueue);
        AddShortcut(Windows.System.VirtualKey.M, Windows.System.VirtualKeyModifiers.Control, ToggleMenu);

        RefreshOverlays();
        RefreshScan();
        RefreshToast();
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

    private void OnToastChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(ToastState.Message))
        {
            RefreshToast();
        }
    }

    private void OnSelectionChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName is nameof(AlbumSelection.Index) or nameof(AlbumSelection.IsActive))
        {
            CollectionHeader.Visibility = State.Selection.IsActive
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

    private void RefreshToast()
    {
        var message = State.Toast.Message;
        Toast.Visibility = message is null ? Visibility.Collapsed : Visibility.Visible;
        ToastText.Text = message ?? string.Empty;
    }

    private void OnSearchChanged(Microsoft.UI.Xaml.Controls.AutoSuggestBox sender,
        Microsoft.UI.Xaml.Controls.AutoSuggestBoxTextChangedEventArgs args)
        => State.Selection.Query = sender.Text;

    private void OnOpenFolder(object sender, RoutedEventArgs e) => _ = PickAndScanAsync();

    private async Task PickAndScanAsync()
    {
        var paths = await FolderPicker.PickMusicFoldersAsync(this);
        State.ScanFolders(paths);
        RefreshFolders();
    }

    private void OnOpenSettings(object sender, RoutedEventArgs e) => ShowSettings();

    private void ShowSettings()
    {
        RefreshFolders();
        SettingsDialog.XamlRoot = Content.XamlRoot;
        _ = SettingsDialog.ShowAsync();
    }

    private void OnRescan(object sender, RoutedEventArgs e) => State.ScanSavedFolders();

    private void OnRemoveFolder(object sender, RoutedEventArgs e)
    {
        if (((FrameworkElement)sender).Tag is string path)
        {
            State.RemoveFolder(path);
            RefreshFolders();
        }
    }

    private void RefreshFolders()
    {
        var empty = State.Settings.MusicFolders.Count == 0;
        NoFolders.Visibility = empty ? Visibility.Visible : Visibility.Collapsed;
        RescanButton.IsEnabled = !empty;
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

    private void OnMenuOpening(Microsoft.UI.Xaml.Controls.SplitView sender, object args)
        => PlayAllButton.IsEnabled = State.Library.Tracks.Count > 0;

    private void OnQueuePaneOpening(Microsoft.UI.Xaml.Controls.SplitView sender, object args)
        => State.Queue.IsOpen = true;

    private void OnQueuePaneClosed(Microsoft.UI.Xaml.Controls.SplitView sender, object args)
        => State.Queue.IsOpen = false;

    private void OnMenuPlayAll(object sender, RoutedEventArgs e)
    {
        MenuPane.IsPaneOpen = false;
        State.PlayAll();
    }

    private void OnMenuPlayScreen(object sender, RoutedEventArgs e)
    {
        MenuPane.IsPaneOpen = false;
        State.Playback.ShowPlayView = true;
    }

    private void OnMenuSettings(object sender, RoutedEventArgs e)
    {
        MenuPane.IsPaneOpen = false;
        ShowSettings();
    }
}
