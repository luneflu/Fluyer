using System.Collections.ObjectModel;
using Fluyer.Core.Native;
using Fluyer.Core.State;
using Fluyer.Shared;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Layout = Fluyer.Shared.Layout;

namespace Fluyer.Screens;

/// <summary>
/// Right sidebar content: queue rows, goto / reorder / remove, clear. Port of
/// macOS <c>QueueView</c> + <c>QueueRow</c>; rows share the song grid's cover key.
/// </summary>
public sealed partial class QueueView : UserControl
{
    public static readonly DependencyProperty QueueProperty =
        DependencyProperty.Register(nameof(Queue), typeof(QueueState), typeof(QueueView),
            new PropertyMetadata(null, OnQueueStateChanged));

    public static readonly DependencyProperty LibraryProperty =
        DependencyProperty.Register(nameof(Library), typeof(LibraryState), typeof(QueueView),
            new PropertyMetadata(null));

    public static readonly DependencyProperty CoversProperty =
        DependencyProperty.Register(nameof(Covers), typeof(CoverState), typeof(QueueView),
            new PropertyMetadata(null));

    public QueueState? Queue
    {
        get => (QueueState?)GetValue(QueueProperty);
        set => SetValue(QueueProperty, value);
    }

    /// <summary>Resolves a queue row to its library index (covers are keyed by library index).</summary>
    public LibraryState? Library
    {
        get => (LibraryState?)GetValue(LibraryProperty);
        set => SetValue(LibraryProperty, value);
    }

    public CoverState? Covers
    {
        get => (CoverState?)GetValue(CoversProperty);
        set => SetValue(CoversProperty, value);
    }

    private static int Pixels => (int)Layout.Number("TrackRowCoverPixels");

    /// <summary>
    /// Mirror of <see cref="QueueState.Tracks"/>: ListView reordering needs a
    /// mutable observable source. A drop moves the row here first, then the
    /// engine; the reload that follows rebuilds this list from the engine.
    /// </summary>
    private readonly ObservableCollection<TrackItemViewModel> Items = new();

    public QueueView()
    {
        InitializeComponent();
        Rows.ItemsSource = Items;
        Rows.ContainerContentChanging += OnContainerChanging;
    }

    public static Windows.UI.Text.FontWeight RowWeight(bool isCurrent)
        => isCurrent ? Microsoft.UI.Text.FontWeights.SemiBold : Microsoft.UI.Text.FontWeights.Normal;

    private static void OnQueueStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (QueueView)d;
        if (e.OldValue is QueueState oldQueue)
        {
            oldQueue.PropertyChanged -= self.OnQueueChanged;
        }
        if (e.NewValue is QueueState newQueue)
        {
            newQueue.PropertyChanged += self.OnQueueChanged;
        }
        self.RefreshEmpty();
    }

    private void OnQueueChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(QueueState.Tracks))
        {
            RefreshEmpty();
        }
    }

    private void RefreshEmpty()
    {
        Items.Clear();
        foreach (var track in Queue?.Tracks ?? Array.Empty<TrackItemViewModel>())
        {
            Items.Add(track);
        }
        var empty = (Queue?.Tracks.Count ?? 0) == 0;
        EmptyLabel.Visibility = empty ? Visibility.Visible : Visibility.Collapsed;
        ClearButton.IsEnabled = !empty;
    }

    private void OnContainerChanging(ListViewBase sender, ContainerContentChangingEventArgs args)
    {
        if (args.Item is not TrackItemViewModel track || args.ItemContainer is not ListViewItem container)
        {
            return;
        }
        if (CoverImages.FindChild<Grid>(container) is Grid row)
        {
            SetHover(row, false);
        }
        // ponytail: linear path lookup per realized row, like macOS; index by path if queues get huge.
        var thumb = CoverImages.FindChild<Image>(container);
        var libraryIndex = Library?.Tracks.FirstOrDefault(t => t.Path == track.Path)?.Index;
        if (thumb is not null && libraryIndex is { } index && Covers is { } covers)
        {
            _ = covers.LoadTrack(thumb, index, Pixels, track);
        }
    }

    // Hover: cover becomes a jump button, duration swaps for remove (not on the current row).
    private void OnRowPointerEntered(object sender, PointerRoutedEventArgs e) => SetHover((Grid)sender, true);
    private void OnRowPointerExited(object sender, PointerRoutedEventArgs e) => SetHover((Grid)sender, false);

    private void SetHover(Grid row, bool hovering)
    {
        if (row.DataContext is not TrackItemViewModel track)
        {
            return;
        }
        var show = hovering && !track.IsCurrent;
        var buttons = CoverImages.FindChildren<Button>(row).ToList();
        if (buttons.Count < 2)
        {
            return;
        }
        var (jump, remove) = (buttons[0], buttons[1]);
        jump.Visibility = remove.Visibility = show ? Visibility.Visible : Visibility.Collapsed;
        if (remove.Parent is Grid trailing && trailing.Children[0] is TextBlock duration)
        {
            duration.Opacity = show ? 0 : 0.4;
        }
        if (show && jump.Content is FontIcon icon)
        {
            // Backward for rows before the playing one, forward otherwise.
            var current = CurrentRow();
            icon.Glyph = current is { } c && (int)track.Index < c ? "\uE892" : "\uE893";
        }
    }

    private int? CurrentRow()
    {
        var tracks = Queue?.Tracks;
        if (tracks is null)
        {
            return null;
        }
        for (var i = 0; i < tracks.Count; i++)
        {
            if (tracks[i].IsCurrent)
            {
                return i;
            }
        }
        return null;
    }

    // Row Index is the queue position before the drag; its new slot in Items is the target.
    private void OnDragCompleted(ListViewBase sender, DragItemsCompletedEventArgs args)
    {
        if (args.Items.FirstOrDefault() is not TrackItemViewModel track)
        {
            return;
        }
        var from = (int)track.Index;
        var to = Items.IndexOf(track);
        if (to < 0 || to == from)
        {
            return;
        }
        Queue?.Move(from, to - from);
    }

    private void OnItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is TrackItemViewModel t)
        {
            Queue?.Goto((int)t.Index);
        }
    }

    private static int RowIndex(object sender) => (int)(ulong)((FrameworkElement)sender).Tag;

    private void OnPlay(object sender, RoutedEventArgs e) => Queue?.Goto(RowIndex(sender));
    private void OnRemove(object sender, RoutedEventArgs e) => Queue?.Remove(RowIndex(sender));
    private void OnClear(object sender, RoutedEventArgs e) => Queue?.Clear();
}
