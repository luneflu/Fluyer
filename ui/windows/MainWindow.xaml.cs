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

    public MainWindow(AppState state)
    {
        State = state;
        InitializeComponent();

        ExtendsContentIntoTitleBar = true;
        SetTitleBar(AppTitleBar);
        AppWindow.SetIcon("Assets/AppIcon.ico");
        AppWindow.Resize(new SizeInt32(1050, 720));

        State.Playback.PropertyChanged += OnPlaybackChanged;
        State.Library.PropertyChanged += OnLibraryChanged;
        State.Toast.PropertyChanged += OnToastChanged;
        State.Selection.PropertyChanged += OnSelectionChanged;
        MusicGrid.OpenFolderRequested += () => _ = PickAndScanAsync();

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
        NowPlaying.Visibility = showing ? Visibility.Visible : Visibility.Collapsed;
        LibraryRoot.Visibility = showing ? Visibility.Collapsed : Visibility.Visible;
    }

    private void RefreshScan()
    {
        var status = State.Library.ScanStatus;
        ScanStatus.Visibility = status.IsScanning ? Visibility.Visible : Visibility.Collapsed;
        OpenFolderButton.Visibility = status.IsScanning ? Visibility.Collapsed : Visibility.Visible;
        ScanLabel.Text = status.StatusLabel;
    }

    private void RefreshToast()
    {
        var message = State.Toast.Message;
        Toast.Visibility = message is null ? Visibility.Collapsed : Visibility.Visible;
        ToastText.Text = message ?? string.Empty;
    }

    private void OnOpenFolder(object sender, RoutedEventArgs e) => _ = PickAndScanAsync();

    private async Task PickAndScanAsync()
    {
        var paths = await FolderPicker.PickMusicFoldersAsync(this);
        State.ScanFolders(paths);
    }
}
