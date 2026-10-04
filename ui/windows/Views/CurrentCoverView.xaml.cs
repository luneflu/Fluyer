using Fluyer.Core.State;
using Fluyer.Core.Support;
using Fluyer.Support;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Views;

/// <summary>
/// Shared cover-art image. Loading is keyed on the playing track path so the
/// bitmap is fetched once per song change and reused by both surfaces.
/// </summary>
public sealed partial class CurrentCoverView : UserControl
{
    public static readonly DependencyProperty SideProperty =
        DependencyProperty.Register(nameof(Side), typeof(double), typeof(CurrentCoverView),
            new PropertyMetadata(40.0, OnSideChanged));

    public static readonly DependencyProperty StateProperty =
        DependencyProperty.Register(nameof(State), typeof(AppState), typeof(CurrentCoverView),
            new PropertyMetadata(null, OnStateChanged));

    public double Side
    {
        get => (double)GetValue(SideProperty);
        set => SetValue(SideProperty, value);
    }

    public AppState? State
    {
        get => (AppState?)GetValue(StateProperty);
        set => SetValue(StateProperty, value);
    }

    public CurrentCoverView()
    {
        InitializeComponent();
        Loaded += (_, _) => Refresh();
    }

    private static void OnSideChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (CurrentCoverView)d;
        self.Root.Width = self.Side;
        self.Root.Height = self.Side;
        self.Placeholder.FontSize = self.Side * 0.4;
        self.Refresh();
    }

    private static void OnStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (CurrentCoverView)d;
        if (e.OldValue is AppState oldState)
        {
            oldState.Playback.PropertyChanged -= self.OnPlaybackChanged;
        }
        if (e.NewValue is AppState newState)
        {
            newState.Playback.PropertyChanged += self.OnPlaybackChanged;
        }
        self.Refresh();
    }

    private void OnPlaybackChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(Core.State.PlaybackState.PlayView))
        {
            Refresh();
        }
    }

    private async void Refresh()
    {
        var state = State;
        if (state is null)
        {
            return;
        }
        Root.Width = Side;
        Root.Height = Side;
        Placeholder.FontSize = Side * 0.4;

        var path = state.Playback.PlayView.Track?.Path ?? "none";
        var pixels = Math.Max(32, (int)(Side * 2));
        var key = ThumbnailKey.Current(pixels, path);
        var engine = state.Engine;
        if (engine is null)
        {
            return;
        }
        await CoverImages.LoadInto(Cover, key, path, () => engine.GetCurrentThumbnail((uint)pixels));
        var hasArt = Cover.Source is not null;
        Placeholder.Visibility = hasArt ? Visibility.Collapsed : Visibility.Visible;
        Cover.Opacity = hasArt ? 1 : 0;
    }
}
