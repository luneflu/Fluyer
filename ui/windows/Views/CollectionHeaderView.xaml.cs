using Fluyer.Core.State;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Views;

/// <summary>Title strip and album actions shown while an album is selected.</summary>
public sealed partial class CollectionHeaderView : UserControl
{
    public static readonly DependencyProperty StateProperty =
        DependencyProperty.Register(nameof(State), typeof(AppState), typeof(CollectionHeaderView),
            new PropertyMetadata(null));

    public AppState? State
    {
        get => (AppState?)GetValue(StateProperty);
        set => SetValue(StateProperty, value);
    }

    public CollectionHeaderView()
    {
        InitializeComponent();
    }

    private void OnBack(object sender, RoutedEventArgs e) => State?.Selection.Clear();
    private void OnPlay(object sender, RoutedEventArgs e) => State?.Selection.PlaySelected();
    private void OnQueue(object sender, RoutedEventArgs e) => State?.Selection.QueueSelected();
    private void OnShuffle(object sender, RoutedEventArgs e) => State?.Selection.ShuffleSelected();
}
