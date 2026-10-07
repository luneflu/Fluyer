using Fluyer.Core.State;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Views;

/// <summary>Left sidebar content: play all, play screen, settings.</summary>
public sealed partial class MenuPaneView : UserControl
{
    public static readonly DependencyProperty StateProperty =
        DependencyProperty.Register(nameof(State), typeof(AppState), typeof(MenuPaneView),
            new PropertyMetadata(null, OnStateChanged));

    public AppState? State
    {
        get => (AppState?)GetValue(StateProperty);
        set => SetValue(StateProperty, value);
    }

    /// <summary>Raised after an action that should close the sidebar.</summary>
    public event Action? CloseRequested;

    /// <summary>Raised by the settings row; the window owns the dialog.</summary>
    public event Action? SettingsRequested;

    public MenuPaneView() => InitializeComponent();

    private static void OnStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (MenuPaneView)d;
        if (e.OldValue is AppState oldState)
        {
            oldState.Library.Tracks.CollectionChanged -= self.OnTracksChanged;
        }
        if (e.NewValue is AppState newState)
        {
            newState.Library.Tracks.CollectionChanged += self.OnTracksChanged;
        }
        self.RefreshPlayAll();
    }

    private void OnTracksChanged(object? sender, System.Collections.Specialized.NotifyCollectionChangedEventArgs e)
        => RefreshPlayAll();

    private void RefreshPlayAll() => PlayAllButton.IsEnabled = (State?.Library.Tracks.Count ?? 0) > 0;

    private void OnPlayAll(object sender, RoutedEventArgs e)
    {
        CloseRequested?.Invoke();
        State?.PlayAll();
    }

    private void OnPlayScreen(object sender, RoutedEventArgs e)
    {
        CloseRequested?.Invoke();
        if (State is not null)
        {
            State.Playback.ShowPlayView = true;
        }
    }

    private void OnSettings(object sender, RoutedEventArgs e)
    {
        CloseRequested?.Invoke();
        SettingsRequested?.Invoke();
    }
}
