using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;

namespace Fluyer.Controls;

/// <summary>
/// Draggable horizontal slider. Port of <c>SliderTrack</c>
/// (<c>ui/macos/Sources/Views/Shared/SliderTrack.swift</c>).
/// </summary>
public sealed partial class SliderTrack : UserControl
{
    public static readonly DependencyProperty FractionProperty =
        DependencyProperty.Register(
            nameof(Fraction), typeof(double), typeof(SliderTrack),
            new PropertyMetadata(0.0, OnFractionPropertyChanged));

    public static readonly DependencyProperty IsThinProperty =
        DependencyProperty.Register(
            nameof(IsThin), typeof(bool), typeof(SliderTrack),
            new PropertyMetadata(false, OnIsThinPropertyChanged));

    /// <summary>0..1 progress. Set by binding; read by the host via <see cref="UserChanged"/>.</summary>
    public double Fraction
    {
        get => (double)GetValue(FractionProperty);
        set => SetValue(FractionProperty, value);
    }

    public bool IsThin
    {
        get => (bool)GetValue(IsThinProperty);
        set => SetValue(IsThinProperty, value);
    }

    /// <summary>Raised only for user drags (never for binding pushes).</summary>
    public event Action<double>? UserChanged;

    private bool _dragging;
    private bool _hovering;

    public SliderTrack()
    {
        InitializeComponent();
        Root.PointerPressed += OnPointerPressed;
        Root.PointerMoved += OnPointerMoved;
        Root.PointerReleased += OnPointerReleased;
        Root.PointerCanceled += OnPointerReleased;
        Root.PointerEntered += (_, _) => { _hovering = true; UpdateVisuals(); };
        Root.PointerExited += (_, _) => { _hovering = false; UpdateVisuals(); };
        Root.SizeChanged += (_, _) => UpdateVisuals();
        UpdateVisuals();
    }

    private static void OnFractionPropertyChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
        => ((SliderTrack)d).UpdateVisuals();

    private static void OnIsThinPropertyChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
        => ((SliderTrack)d).UpdateVisuals();

    private double Thickness => IsThin ? (_hovering || _dragging ? 5 : 4) : (_hovering || _dragging ? 6 : 5);

    private void UpdateVisuals()
    {
        var thickness = Thickness;
        Back.Height = thickness;
        Back.CornerRadius = new CornerRadius(thickness / 2);
        Front.Height = thickness;
        Front.CornerRadius = new CornerRadius(thickness / 2);
        if (!_dragging)
        {
            var clamped = Math.Min(Math.Max(Fraction, 0.0), 1.0);
            Front.Width = Math.Max(0, Root.ActualWidth * clamped);
        }
    }

    private void OnPointerPressed(object sender, PointerRoutedEventArgs e)
    {
        _dragging = true;
        Root.CapturePointer(e.Pointer);
        Commit(e);
        e.Handled = true;
    }

    private void OnPointerMoved(object sender, PointerRoutedEventArgs e)
    {
        if (_dragging)
        {
            Commit(e);
            e.Handled = true;
        }
    }

    private void OnPointerReleased(object sender, PointerRoutedEventArgs e)
    {
        if (_dragging)
        {
            _dragging = false;
            UpdateVisuals();
            e.Handled = true;
        }
    }

    private void Commit(PointerRoutedEventArgs e)
    {
        // Guard the degenerate zero-width frame: 0/0 is NaN and must never
        // reach the seek conversion (mirrors the Swift guard + clamping).
        if (Root.ActualWidth <= 0)
        {
            return;
        }
        var x = e.GetCurrentPoint(Root).Position.X;
        var fraction = x / Root.ActualWidth;
        if (double.IsNaN(fraction))
        {
            fraction = 0;
        }
        fraction = Math.Min(Math.Max(fraction, 0.0), 1.0);
        Front.Width = Math.Max(0, Root.ActualWidth * fraction);
        // Push back into the DP without echoing through the binding loop: the
        // visual is already exact, so UpdateVisuals skips while dragging.
        SetValue(FractionProperty, fraction);
        UserChanged?.Invoke(fraction);
    }
}
