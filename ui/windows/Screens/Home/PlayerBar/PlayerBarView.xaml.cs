using Fluyer.Core.State;
using Fluyer.Core.Support;
using Fluyer.Shared;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Screens;

/// <summary>
/// Always-visible transport strip. Glyphs refresh in code-behind (the icon set
/// is computed from transport state); labels bind straight to the bar snapshot.
/// </summary>
public sealed partial class PlayerBarView : UserControl
{
    public static readonly DependencyProperty PlaybackProperty =
        DependencyProperty.Register(nameof(Playback), typeof(PlaybackState), typeof(PlayerBarView),
            new PropertyMetadata(null, OnPlaybackStateChanged));

    public static readonly DependencyProperty CoversProperty =
        DependencyProperty.Register(nameof(Covers), typeof(CoverState), typeof(PlayerBarView),
            new PropertyMetadata(null));

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

    public PlayerBarView()
    {
        InitializeComponent();
        SeekBar.UserChanged += fraction => Playback?.SeekFraction((float)fraction);
        VolumeBar.UserChanged += fraction => Playback?.SetVolume((float)fraction);
    }

    private static void OnPlaybackStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (PlayerBarView)d;
        if (e.OldValue is PlaybackState oldPlayback)
        {
            oldPlayback.PropertyChanged -= self.OnPlaybackChanged;
        }
        if (e.NewValue is PlaybackState newPlayback)
        {
            newPlayback.PropertyChanged += self.OnPlaybackChanged;
        }
        self.RefreshIcons();
    }

    private void OnPlaybackChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName is nameof(PlaybackState.PlayerBar) or nameof(PlaybackState.Bar))
        {
            RefreshIcons();
        }
    }

    private void RefreshIcons()
    {
        var bar = Playback?.PlayerBar;
        if (bar is null)
        {
            return;
        }
        PlayPauseIcon.Glyph = bar.IsPlaying ? "\uE769" : "\uE768";
        RepeatIcon.Glyph = PlaybackIcons.RepeatIcon(bar.RepeatMode);
        RepeatIcon.Opacity = bar.RepeatMode == Core.Native.RepeatMode.None ? 0.5 : 1;
        ShuffleIcon.Opacity = bar.IsShuffled ? 1 : 0.5;
        VolumeIcon.Glyph = PlaybackIcons.Volume(bar.Volume);
        VolumeIcon.Opacity = bar.Volume > 0.001 ? 1 : 0.5;
    }

    private void OnPrevious(object sender, RoutedEventArgs e) => Playback?.Previous();
    private void OnTogglePlay(object sender, RoutedEventArgs e) => Playback?.TogglePlay();
    private void OnNext(object sender, RoutedEventArgs e) => Playback?.Next();
    private void OnCycleRepeat(object sender, RoutedEventArgs e) => Playback?.CycleRepeat();
    private void OnShuffle(object sender, RoutedEventArgs e) => Playback?.Shuffle();
    private void OnToggleMute(object sender, RoutedEventArgs e) => Playback?.ToggleMute();
    private void OnOpenNowPlaying(object sender, RoutedEventArgs e)
    {
        if (Playback is not null)
        {
            Playback.ShowPlayView = true;
        }
    }
}
