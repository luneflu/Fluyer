using Fluyer.Core.State;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Screens;

/// <summary>Title strip and album actions shown while an album is selected.</summary>
public sealed partial class AlbumHeaderView : UserControl
{
    public static readonly DependencyProperty FilterProperty =
        DependencyProperty.Register(nameof(Filter), typeof(LibraryFilterState), typeof(AlbumHeaderView),
            new PropertyMetadata(null));

    public LibraryFilterState? Filter
    {
        get => (LibraryFilterState?)GetValue(FilterProperty);
        set => SetValue(FilterProperty, value);
    }

    public AlbumHeaderView()
    {
        InitializeComponent();
    }

    private void OnBack(object sender, RoutedEventArgs e) => Filter?.Clear();
    private void OnPlay(object sender, RoutedEventArgs e) => Filter?.PlaySelected();
    private void OnQueue(object sender, RoutedEventArgs e) => Filter?.QueueSelected();
    private void OnShuffle(object sender, RoutedEventArgs e) => Filter?.ShuffleSelected();
}
