using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Core.Support;
using Fluyer.Shared;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Layout = Fluyer.Shared.Layout;
using Microsoft.UI.Xaml.Input;

namespace Fluyer.Screens;

/// <summary>
/// Full-screen now-playing surface.
/// </summary>
public sealed partial class PlayView : UserControl
{
    public static readonly DependencyProperty PlaybackProperty =
        DependencyProperty.Register(nameof(Playback), typeof(PlaybackState), typeof(PlayView),
            new PropertyMetadata(null, OnPlaybackStateChanged));

    public static readonly DependencyProperty CoversProperty =
        DependencyProperty.Register(nameof(Covers), typeof(CoverState), typeof(PlayView),
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

    private Microsoft.UI.Dispatching.DispatcherQueueTimer? _idleTimer;

    public PlayView()
    {
        InitializeComponent();
        Progress.UserChanged += fraction => Playback?.SeekFraction((float)fraction);
        Volume.UserChanged += fraction => Playback?.SetVolume((float)fraction);
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

    private static void OnPlaybackStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (PlayView)d;
        if (e.OldValue is PlaybackState oldPlayback)
        {
            oldPlayback.PropertyChanged -= self.OnPlaybackChanged;
            oldPlayback.Clock.PropertyChanged -= self.OnClockChanged;
        }
        if (e.NewValue is PlaybackState newPlayback)
        {
            newPlayback.PropertyChanged += self.OnPlaybackChanged;
            newPlayback.Clock.PropertyChanged += self.OnClockChanged;
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

    private bool HasLyrics => (Playback?.PlayView.Lyrics.Count ?? 0) > 1;

    private void LayoutColumns()
    {
        if (Playback is null)
        {
            return;
        }
        var has = HasLyrics;
        Lyrics.Visibility = has ? Visibility.Visible : Visibility.Collapsed;
        LyricsColumn.Width = has
            ? (GridLength)Application.Current.Resources["PlayLyricsColumnWidth"]
            : new GridLength(0);
        LeftColumn.Width = new GridLength(1, GridUnitType.Star);
        // Swift: trailing-aligned against lyrics with a gutter, centered without.
        var gutter = Layout.Number("PlayColumnGutter");
        LeftStack.HorizontalAlignment = has ? HorizontalAlignment.Right : HorizontalAlignment.Center;
        LeftStack.Margin = has ? new Thickness(0, 0, gutter, 0) : new Thickness(0);

        var ratio = Layout.Number(has ? "PlayCoverColumnRatio" : "PlayCenteredColumnRatio");
        var columnWidth = ActualWidth > 0 ? ActualWidth * ratio : 400;
        var side = Math.Min(Math.Max(columnWidth - gutter, Layout.Number("PlayMinCoverSide")),
            Layout.Number("PlayMaxCoverSide"));
        Cover.Side = side;
        ControlCard.Width = side;
    }

    // Sequence guard: clock ticks fire faster than the settle delay below,
    // so a stale continuation must not fight the newest glide.
    private int _lyricScrollSeq;

    private async void HighlightActiveLyric()
    {
        var playback = Playback;
        if (playback is null)
        {
            return;
        }
        var lyrics = playback.PlayView.Lyrics;
        var active = playback.Clock.CurrentLyricIndex;
        foreach (var item in Lyrics.Items)
        {
            if (Lyrics.ContainerFromItem(item) is ListViewItem container
                && item is LyricLine line
                && ResolveLyricParts(container, out var note, out var text)
                && text is not null)
            {
                var index = lyrics.IndexOf(line);
                var isActive = index == active;
                if (string.IsNullOrEmpty(line.Text))
                {
                    if (note is not null)
                    {
                        note.FontSize = isActive ? 33 : 27;
                        note.Opacity = isActive ? 0.95 : 0.5;
                    }
                }
                else
                {
                    text.Opacity = isActive ? 1 : 0.4;
                    text.FontSize = isActive ? 30 : 25;
                }
            }
        }
        if (active >= 0 && active < lyrics.Count)
        {
            // Swift's proxy.scrollTo(anchor: .center) is ONE animated glide
            // from the current offset. The old code called ScrollIntoView
            // first (instant snap to top-aligned) then ChangeView — that
            // snap-then-glide is the 0-to-target weirdness. Now: only ever
            // ChangeView from wherever the scroller already sits.
            var seq = ++_lyricScrollSeq;
            await Task.Delay(50); // let the FontSize re-layout settle
            if (seq != _lyricScrollSeq)
            {
                return; // superseded by a newer tick
            }
            if (!TryCenterLyric(active, animate: true) && seq == _lyricScrollSeq)
            {
                // Container virtualized away: realize it with a snap (no
                // glide from a bogus origin), then center without animation.
                Lyrics.ScrollIntoView(lyrics[active], ScrollIntoViewAlignment.Leading);
                await Task.Delay(50);
                if (seq != _lyricScrollSeq)
                {
                    return;
                }
                TryCenterLyric(active, animate: false);
            }
        }
    }

    // Glide/snap the inner ScrollViewer so row <paramref name="active"/> sits
    // at viewport center. False when the container isn't realized (or the
    // scroller isn't found yet) — caller falls back to ScrollIntoView.
    private bool TryCenterLyric(int active, bool animate)
    {
        if (Lyrics.ContainerFromIndex(active) is not ListViewItem activeContainer
            || CoverImages.FindChild<ScrollViewer>(Lyrics) is not { } scroller)
        {
            return false;
        }
        var itemPos = activeContainer.TransformToVisual(scroller)
            .TransformPoint(new Windows.Foundation.Point(0, 0));
        var target = scroller.VerticalOffset + itemPos.Y
            - scroller.ViewportHeight / 2
            + activeContainer.ActualHeight / 2;
        scroller.ChangeView(null, Math.Max(0, target), null, disableAnimation: !animate);
        return true;
    }

    // Template parts for one lyric row: the note placeholder (instrumental
    // gaps) and the lyric text. True when at least one part resolved.
    private static bool ResolveLyricParts(
        DependencyObject container, out FontIcon? note, out TextBlock? text)
    {
        note = null;
        text = null;
        foreach (var icon in CoverImages.FindChildren<FontIcon>(container))
        {
            note = icon;
            break;
        }
        text = CoverImages.FindChild<TextBlock>(container);
        return note is not null || text is not null;
    }

    private void OnLyricContainerChanging(ListViewBase sender, ContainerContentChangingEventArgs args)
    {
        if (args.Item is LyricLine line
            && args.ItemContainer is ListViewItem container
            && ResolveLyricParts(container, out var note, out var text)
            && text is not null)
        {
            // Swift uses the same music.note placeholder for instrumental gaps.
            var empty = string.IsNullOrEmpty(line.Text);
            var lyrics = Playback?.PlayView.Lyrics;
            var isActive = lyrics is not null && Playback is not null
                && lyrics.IndexOf(line) == Playback.Clock.CurrentLyricIndex;
            if (note is not null)
            {
                note.Visibility = empty ? Visibility.Visible : Visibility.Collapsed;
                note.FontSize = isActive ? 33 : 27;
                note.Opacity = isActive ? 0.95 : 0.5;
            }
            text.Visibility = empty ? Visibility.Collapsed : Visibility.Visible;
            text.Text = empty ? string.Empty : line.Text;
            text.Opacity = isActive ? 1 : 0.4;
            text.FontSize = isActive ? 30 : 25;
        }
    }

    private void OnLyricClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is LyricLine line)
        {
            Playback?.Seek(line.TimestampMs);
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
        RepeatIcon.Opacity = bar.RepeatMode == RepeatMode.None ? 0.5 : 1;
        ShuffleIcon.Opacity = bar.IsShuffled ? 1 : 0.5;
        VolumeIcon.Glyph = PlaybackIcons.Volume(bar.Volume);
    }

    private void OnPrevious(object sender, RoutedEventArgs e) => Playback?.Previous();
    private void OnTogglePlay(object sender, RoutedEventArgs e) => Playback?.TogglePlay();
    private void OnNext(object sender, RoutedEventArgs e) => Playback?.Next();
    private void OnCycleRepeat(object sender, RoutedEventArgs e) => Playback?.CycleRepeat();
    private void OnShuffle(object sender, RoutedEventArgs e) => Playback?.Shuffle();
    private void OnToggleMute(object sender, RoutedEventArgs e) => Playback?.ToggleMute();
    private void OnMaxVolume(object sender, RoutedEventArgs e) => Playback?.SetVolume(1.0f);
    private void OnBack(object sender, RoutedEventArgs e) => GoBack();

    private void GoBack()
    {
        if (Playback is not null)
        {
            Playback.ShowPlayView = false;
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
