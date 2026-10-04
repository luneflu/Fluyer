using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Core.Support;
using Fluyer.Support;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;

namespace Fluyer.Views;

/// <summary>
/// Full-screen now-playing surface.
/// </summary>
public sealed partial class PlayView : UserControl
{
    public static readonly DependencyProperty StateProperty =
        DependencyProperty.Register(nameof(State), typeof(AppState), typeof(PlayView),
            new PropertyMetadata(null, OnStateChanged));

    public AppState? State
    {
        get => (AppState?)GetValue(StateProperty);
        set => SetValue(StateProperty, value);
    }

    private Microsoft.UI.Dispatching.DispatcherQueueTimer? _idleTimer;

    public PlayView()
    {
        InitializeComponent();
        Progress.UserChanged += fraction => State?.Playback.SeekFraction((float)fraction);
        Volume.UserChanged += fraction => State?.Playback.SetVolume((float)fraction);
        Lyrics.ContainerContentChanging += OnLyricContainerChanging;
        SizeChanged += (_, _) => LayoutColumns();
        Root.PointerMoved += (_, _) => ResetIdleTimer();

        var esc = new KeyboardAccelerator { Key = Windows.System.VirtualKey.Escape };
        esc.Invoked += (_, _) => GoBack();
        BackButton.KeyboardAccelerators.Add(esc);
    }

    // x:Bind function bindings (OneWay — re-evaluated when PlayerBar raises).
    public string TimeFormatElapsed(ulong ms) => TimeFormat.Elapsed(ms);

    public string TrackLabel(PlayerBarViewModel bar)
    {
        var artist = string.IsNullOrEmpty(bar.Artist) ? "Fluyer" : bar.Artist;
        var title = string.IsNullOrEmpty(bar.Title) ? "No track playing" : bar.Title;
        return $"{artist} • {title}";
    }

    private static void OnStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (PlayView)d;
        if (e.OldValue is AppState oldState)
        {
            oldState.Playback.PropertyChanged -= self.OnPlaybackChanged;
            oldState.Playback.Clock.PropertyChanged -= self.OnClockChanged;
        }
        if (e.NewValue is AppState newState)
        {
            newState.Playback.PropertyChanged += self.OnPlaybackChanged;
            newState.Playback.Clock.PropertyChanged += self.OnClockChanged;
        }
        self.LayoutColumns();
        self.RefreshIcons();
        self.ResetIdleTimer();
    }

    private void OnPlaybackChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName is nameof(PlaybackState.PlayView))
        {
            LayoutColumns();
        }
        else if (e.PropertyName is nameof(PlaybackState.PlayerBar) or nameof(PlaybackState.Bar))
        {
            RefreshIcons();
        }
    }

    private void OnClockChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(PlaybackClock.CurrentLyricIndex))
        {
            HighlightActiveLyric();
        }
    }

    private bool HasLyrics => (State?.Playback.PlayView.Lyrics.Count ?? 0) > 1;

    private void LayoutColumns()
    {
        if (State is null)
        {
            return;
        }
        var has = HasLyrics;
        Lyrics.Visibility = has ? Visibility.Visible : Visibility.Collapsed;
        LyricsColumn.Width = has ? new GridLength(1.375, GridUnitType.Star) : new GridLength(0);
        LeftColumn.Width = new GridLength(1, GridUnitType.Star);

        var columnWidth = ActualWidth > 0 ? ActualWidth * (has ? 0.40 : 0.50) : 400;
        var side = Math.Min(Math.Max(columnWidth - 40, 200), 360);
        Cover.Side = side;
        ControlCard.Width = side;
    }

    private void HighlightActiveLyric()
    {
        var state = State;
        if (state is null)
        {
            return;
        }
        var lyrics = state.Playback.PlayView.Lyrics;
        var active = state.Playback.Clock.CurrentLyricIndex;
        foreach (var item in Lyrics.Items)
        {
            if (Lyrics.ContainerFromItem(item) is ListViewItem container
                && CoverImages.FindChild<TextBlock>(container) is { } text)
            {
                var index = lyrics.IndexOf((LyricLine)item);
                var isActive = index == active;
                text.Opacity = isActive ? 1 : 0.4;
                text.FontSize = isActive ? 30 : 25;
            }
        }
        if (active >= 0 && active < lyrics.Count)
        {
            Lyrics.ScrollIntoView(lyrics[active]);
        }
    }

    private void OnLyricContainerChanging(ListViewBase sender, ContainerContentChangingEventArgs args)
    {
        if (args.Item is LyricLine line
            && args.ItemContainer is ListViewItem container
            && CoverImages.FindChild<TextBlock>(container) is { } text)
        {
            if (string.IsNullOrEmpty(line.Text))
            {
                text.Text = "♪";
            }
            var state = State;
            if (state is not null)
            {
                var isActive = state.Playback.PlayView.Lyrics.IndexOf(line) == state.Playback.Clock.CurrentLyricIndex;
                text.Opacity = isActive ? 1 : 0.4;
                text.FontSize = isActive ? 30 : 25;
            }
        }
    }

    private void OnLyricClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is LyricLine line)
        {
            State?.Playback.Seek(line.TimestampMs);
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
        RepeatIcon.Opacity = bar.RepeatMode == RepeatMode.None ? 0.5 : 1;
        ShuffleIcon.Opacity = bar.IsShuffled ? 1 : 0.5;
        VolumeIcon.Glyph = PlaybackIcons.Volume(bar.Volume);
    }

    private void OnPrevious(object sender, RoutedEventArgs e) => State?.Playback.Previous();
    private void OnTogglePlay(object sender, RoutedEventArgs e) => State?.Playback.TogglePlay();
    private void OnNext(object sender, RoutedEventArgs e) => State?.Playback.Next();
    private void OnCycleRepeat(object sender, RoutedEventArgs e) => State?.Playback.CycleRepeat();
    private void OnShuffle(object sender, RoutedEventArgs e) => State?.Playback.Shuffle();
    private void OnToggleMute(object sender, RoutedEventArgs e) => State?.Playback.ToggleMute();
    private void OnMaxVolume(object sender, RoutedEventArgs e) => State?.Playback.SetVolume(1.0f);
    private void OnBack(object sender, RoutedEventArgs e) => GoBack();

    private void GoBack()
    {
        if (State is not null)
        {
            State.Playback.ShowPlayView = false;
        }
    }

    private void ResetIdleTimer()
    {
        BackButton.Opacity = 0.7;
        _idleTimer?.Stop();
        var queue = Microsoft.UI.Dispatching.DispatcherQueue.GetForCurrentThread();
        _idleTimer = queue.CreateTimer();
        _idleTimer.Interval = TimeSpan.FromSeconds(3);
        _idleTimer.Tick += (_, _) =>
        {
            BackButton.Opacity = 0;
        };
        _idleTimer.Start();
    }
}
