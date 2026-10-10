using Fluyer.Core.State;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Screens;

/// <summary>
/// Settings dialog body. Takes the whole <see cref="AppState"/> like macOS
/// <c>SettingsView</c>: folder add/remove/rescan span settings and library.
/// </summary>
public sealed partial class SettingsView : UserControl
{
    public AppState? State { get; set; }

    /// <summary>Raised by "Add folder..."; the window owns the picker (needs its HWND).</summary>
    public event Action? AddFolderRequested;

    public SettingsView() => InitializeComponent();

    /// <summary>Re-read the folder list state (empty label, rescan button).</summary>
    public void Refresh()
    {
        var empty = (State?.Settings.MusicFolders.Count ?? 0) == 0;
        NoFolders.Visibility = empty ? Visibility.Visible : Visibility.Collapsed;
        RescanButton.IsEnabled = !empty;
    }

    private void OnAddFolder(object sender, RoutedEventArgs e) => AddFolderRequested?.Invoke();

    private void OnRescan(object sender, RoutedEventArgs e) => State?.ScanSavedFolders();

    private void OnRemoveFolder(object sender, RoutedEventArgs e)
    {
        if (((FrameworkElement)sender).Tag is string path)
        {
            State?.RemoveFolder(path);
            Refresh();
        }
    }
}
