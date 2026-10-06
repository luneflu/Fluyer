using Fluyer.Core.Native;
using Fluyer.Core.State;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Views;

/// <summary>Right sidebar content: queue rows, goto / reorder / remove, clear.</summary>
public sealed partial class QueuePaneView : UserControl
{
    public static readonly DependencyProperty StateProperty =
        DependencyProperty.Register(nameof(State), typeof(AppState), typeof(QueuePaneView),
            new PropertyMetadata(null, OnStateChanged));

    public AppState? State
    {
        get => (AppState?)GetValue(StateProperty);
        set => SetValue(StateProperty, value);
    }

    public QueuePaneView() => InitializeComponent();

    public static Windows.UI.Text.FontWeight RowWeight(bool isCurrent)
        => isCurrent ? Microsoft.UI.Text.FontWeights.Bold : Microsoft.UI.Text.FontWeights.Normal;

    private static void OnStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (QueuePaneView)d;
        if (e.OldValue is AppState oldState)
        {
            oldState.Queue.PropertyChanged -= self.OnQueueChanged;
        }
        if (e.NewValue is AppState newState)
        {
            newState.Queue.PropertyChanged += self.OnQueueChanged;
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
        var empty = (State?.Queue.Tracks.Count ?? 0) == 0;
        EmptyLabel.Visibility = empty ? Visibility.Visible : Visibility.Collapsed;
        ClearButton.IsEnabled = !empty;
    }

    private void OnItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is TrackItemViewModel t)
        {
            State?.Queue.Goto((int)t.Index);
        }
    }

    private static int RowIndex(object sender) => (int)(ulong)((FrameworkElement)sender).Tag;

    private void OnUp(object sender, RoutedEventArgs e) => State?.Queue.Move(RowIndex(sender), -1);
    private void OnDown(object sender, RoutedEventArgs e) => State?.Queue.Move(RowIndex(sender), 1);
    private void OnRemove(object sender, RoutedEventArgs e) => State?.Queue.Remove(RowIndex(sender));
    private void OnClear(object sender, RoutedEventArgs e) => State?.Queue.Clear();
}
