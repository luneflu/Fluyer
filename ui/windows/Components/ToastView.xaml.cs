using Fluyer.Core.State;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Fluyer.Components;

/// <summary>Shows <see cref="ToastState.Message"/>; hidden while there is none.</summary>
public sealed partial class ToastView : UserControl
{
    public static readonly DependencyProperty ToastProperty =
        DependencyProperty.Register(nameof(Toast), typeof(ToastState), typeof(ToastView),
            new PropertyMetadata(null, OnToastStateChanged));

    public ToastState? Toast
    {
        get => (ToastState?)GetValue(ToastProperty);
        set => SetValue(ToastProperty, value);
    }

    public ToastView() => InitializeComponent();

    private static void OnToastStateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        var self = (ToastView)d;
        if (e.OldValue is ToastState oldToast)
        {
            oldToast.PropertyChanged -= self.OnToastChanged;
        }
        if (e.NewValue is ToastState newToast)
        {
            newToast.PropertyChanged += self.OnToastChanged;
        }
        self.Refresh();
    }

    private void OnToastChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(ToastState.Message))
        {
            Refresh();
        }
    }

    private void Refresh()
    {
        var message = Toast?.Message;
        Visibility = message is null ? Visibility.Collapsed : Visibility.Visible;
        ToastText.Text = message ?? string.Empty;
    }
}
