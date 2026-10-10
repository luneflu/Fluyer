using Fluyer.Core.State;
using Fluyer.Shared;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Components;

/// <summary>
/// Shared cover-art image. Loading is keyed on the playing track path so the
/// bitmap is fetched once per song change and reused by both surfaces.
/// </summary>
public sealed partial class CurrentCoverView : UserControl
{
    public static readonly DependencyProperty SideProperty =
        DependencyProperty.Register(nameof(Side), typeof(double), typeof(CurrentCoverView),
            new PropertyMetadata(40.0, OnSideChanged));

    public static readonly DependencyProperty PlaybackProperty =
        DependencyProperty.Register(nameof(Playback), typeof(PlaybackState), typeof(CurrentCoverView),
            new PropertyMetadata(null, OnPlaybackStateChanged));

    public double Side
    {
        get => (double)GetValue(SideProperty);
        set => SetValue(SideProperty, value);
    }

    public static readonly DependencyProperty CoversProperty =
        DependencyProperty.Register(nameof(Covers), typeof(CoverState), typeof(CurrentCoverView),
            new PropertyMetadata(null, (d, _) => ((CurrentCoverView)d).Refresh()));

    public CoverState? Covers
    {
        get => (CoverState?)GetValue(CoversProperty);
        set => SetValue(CoversProperty, value);
    }

    public PlaybackState? Playback
    {
        get => (PlaybackState?)GetValue(PlaybackProperty);
        set => SetValue(PlaybackProperty, value);
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

    private static void OnPlaybackStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (CurrentCoverView)d;
        if (e.OldValue is PlaybackState oldPlayback)
        {
            oldPlayback.PropertyChanged -= self.OnPlaybackChanged;
        }
        if (e.NewValue is PlaybackState newPlayback)
        {
            newPlayback.PropertyChanged += self.OnPlaybackChanged;
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
        var playback = Playback;
        var covers = Covers;
        if (playback is null || covers is null)
        {
            return;
        }
        Root.Width = Side;
        Root.Height = Side;
        Placeholder.FontSize = Side * 0.4;

        var path = playback.PlayView.Track?.Path ?? "none";
        var pixels = Math.Max(32, (int)(Side * 2));
        await covers.LoadCurrent(Cover, pixels, path);
        var hasArt = Cover.Source is not null;
        Placeholder.Visibility = hasArt ? Visibility.Collapsed : Visibility.Visible;
        Cover.Opacity = hasArt ? 1 : 0;
    }
}
