using Fluyer.Core.State;
using Fluyer.Core.Support;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Views;

/// <summary>
/// Always-visible transport strip. Glyphs refresh in code-behind (the icon set
/// is computed from transport state); labels bind straight to the bar snapshot.
/// </summary>
public sealed partial class PlayerBarView : UserControl
{
    public static readonly DependencyProperty StateProperty =
        DependencyProperty.Register(nameof(State), typeof(AppState), typeof(PlayerBarView),
            new PropertyMetadata(null, OnStateChanged));

    public AppState? State
    {
        get => (AppState?)GetValue(StateProperty);
        set => SetValue(StateProperty, value);
    }

    public PlayerBarView()
    {
        InitializeComponent();
        SeekBar.UserChanged += fraction => State?.Playback.SeekFraction((float)fraction);
        VolumeBar.UserChanged += fraction => State?.Playback.SetVolume((float)fraction);
    }

    private static void OnStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (PlayerBarView)d;
        if (e.OldValue is AppState oldState)
        {
            oldState.Playback.PropertyChanged -= self.OnPlaybackChanged;
            oldState.Queue.PropertyChanged -= self.OnQueueChanged;
        }
        if (e.NewValue is AppState newState)
        {
            newState.Playback.PropertyChanged += self.OnPlaybackChanged;
            newState.Queue.PropertyChanged += self.OnQueueChanged;
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
        var bar = State?.Playback.PlayerBar;
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

    private void OnPrevious(object sender, RoutedEventArgs e) => State?.Playback.Previous();
    private void OnTogglePlay(object sender, RoutedEventArgs e) => State?.Playback.TogglePlay();
    private void OnNext(object sender, RoutedEventArgs e) => State?.Playback.Next();
    private void OnCycleRepeat(object sender, RoutedEventArgs e) => State?.Playback.CycleRepeat();
    private void OnShuffle(object sender, RoutedEventArgs e) => State?.Playback.Shuffle();
    private void OnToggleMute(object sender, RoutedEventArgs e) => State?.Playback.ToggleMute();
    private void OnOpenNowPlaying(object sender, RoutedEventArgs e)
    {
        if (State is not null)
        {
            State.Playback.ShowPlayView = true;
        }
    }

    // MARK: - Queue flyout

    public static Windows.UI.Text.FontWeight RowWeight(bool isCurrent)
        => isCurrent ? Microsoft.UI.Text.FontWeights.Bold : Microsoft.UI.Text.FontWeights.Normal;

    private void OnQueueOpening(object? sender, object e)
    {
        if (State is null)
        {
            return;
        }
        State.Queue.IsOpen = true;
        RefreshQueueEmpty();
    }

    private void OnQueueClosed(object? sender, object e)
    {
        if (State is not null)
        {
            State.Queue.IsOpen = false;
        }
    }

    private void OnQueueChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(QueueState.Tracks))
        {
            RefreshQueueEmpty();
        }
    }

    private void RefreshQueueEmpty()
        => QueueEmpty.Visibility = State?.Queue.Tracks.Count > 0 ? Visibility.Collapsed : Visibility.Visible;

    private void OnQueueItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is Core.Native.TrackItemViewModel t)
        {
            State?.Queue.Goto((int)t.Index);
        }
    }

    private static int RowIndex(object sender) => (int)(ulong)((FrameworkElement)sender).Tag;

    private void OnQueueUp(object sender, RoutedEventArgs e) => State?.Queue.Move(RowIndex(sender), -1);
    private void OnQueueDown(object sender, RoutedEventArgs e) => State?.Queue.Move(RowIndex(sender), 1);
    private void OnQueueRemove(object sender, RoutedEventArgs e) => State?.Queue.Remove(RowIndex(sender));
}
