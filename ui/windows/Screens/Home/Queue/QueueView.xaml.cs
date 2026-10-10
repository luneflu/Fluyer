using Fluyer.Core.Native;
using Fluyer.Core.State;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Screens;

/// <summary>Right sidebar content: queue rows, goto / reorder / remove, clear.</summary>
public sealed partial class QueueView : UserControl
{
    public static readonly DependencyProperty QueueProperty =
        DependencyProperty.Register(nameof(Queue), typeof(QueueState), typeof(QueueView),
            new PropertyMetadata(null, OnQueueStateChanged));

    public QueueState? Queue
    {
        get => (QueueState?)GetValue(QueueProperty);
        set => SetValue(QueueProperty, value);
    }

    public QueueView() => InitializeComponent();

    public static Windows.UI.Text.FontWeight RowWeight(bool isCurrent)
        => isCurrent ? Microsoft.UI.Text.FontWeights.Bold : Microsoft.UI.Text.FontWeights.Normal;

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
        var empty = (Queue?.Tracks.Count ?? 0) == 0;
        EmptyLabel.Visibility = empty ? Visibility.Visible : Visibility.Collapsed;
        ClearButton.IsEnabled = !empty;
    }

    private void OnItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is TrackItemViewModel t)
        {
            Queue?.Goto((int)t.Index);
        }
    }

    private static int RowIndex(object sender) => (int)(ulong)((FrameworkElement)sender).Tag;

    private void OnUp(object sender, RoutedEventArgs e) => Queue?.Move(RowIndex(sender), -1);
    private void OnDown(object sender, RoutedEventArgs e) => Queue?.Move(RowIndex(sender), 1);
    private void OnRemove(object sender, RoutedEventArgs e) => Queue?.Remove(RowIndex(sender));
    private void OnClear(object sender, RoutedEventArgs e) => Queue?.Clear();
}
