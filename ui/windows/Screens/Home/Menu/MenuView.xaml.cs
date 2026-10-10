using Fluyer.Core.State;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Screens;

/// <summary>Left sidebar content: play all, play screen, settings.</summary>
public sealed partial class MenuView : UserControl
{
    public static readonly DependencyProperty LibraryProperty =
        DependencyProperty.Register(nameof(Library), typeof(LibraryState), typeof(MenuView),
            new PropertyMetadata(null, OnLibraryStateChanged));

    public static readonly DependencyProperty PlaybackProperty =
        DependencyProperty.Register(nameof(Playback), typeof(PlaybackState), typeof(MenuView),
            new PropertyMetadata(null));

    public LibraryState? Library
    {
        get => (LibraryState?)GetValue(LibraryProperty);
        set => SetValue(LibraryProperty, value);
    }

    public PlaybackState? Playback
    {
        get => (PlaybackState?)GetValue(PlaybackProperty);
        set => SetValue(PlaybackProperty, value);
    }

    /// <summary>Raised after an action that should close the sidebar.</summary>
    public event Action? CloseRequested;

    /// <summary>Raised by the settings row; the window owns the dialog.</summary>
    public event Action? SettingsRequested;

    /// <summary>Raised by "Play All"; the window forwards to <see cref="AppState.PlayAll"/>.</summary>
    public event Action? PlayAllRequested;

    public MenuView() => InitializeComponent();

    private static void OnLibraryStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (MenuView)d;
        if (e.OldValue is LibraryState oldLibrary)
        {
            oldLibrary.Tracks.CollectionChanged -= self.OnTracksChanged;
        }
        if (e.NewValue is LibraryState newLibrary)
        {
            newLibrary.Tracks.CollectionChanged += self.OnTracksChanged;
        }
        self.RefreshPlayAll();
    }

    private void OnTracksChanged(object? sender, System.Collections.Specialized.NotifyCollectionChangedEventArgs e)
        => RefreshPlayAll();

    private void RefreshPlayAll() => PlayAllButton.IsEnabled = (Library?.Tracks.Count ?? 0) > 0;

    private void OnPlayAll(object sender, RoutedEventArgs e)
    {
        CloseRequested?.Invoke();
        PlayAllRequested?.Invoke();
    }

    private void OnPlayScreen(object sender, RoutedEventArgs e)
    {
        CloseRequested?.Invoke();
        if (Playback is not null)
        {
            Playback.ShowPlayView = true;
        }
    }

    private void OnSettings(object sender, RoutedEventArgs e)
    {
        CloseRequested?.Invoke();
        SettingsRequested?.Invoke();
    }
}
