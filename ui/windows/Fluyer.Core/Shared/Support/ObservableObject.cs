using System.ComponentModel;
using System.Runtime.CompilerServices;

namespace Fluyer.Core.Support;

/// <summary>
/// Minimal <see cref="INotifyPropertyChanged"/> base. Hand-rolled (rather than
/// CommunityToolkit.Mvvm) so <c>Fluyer.Core</c> stays dependency-free and the
/// state layer remains usable from the plain xUnit project.
/// </summary>
public abstract class ObservableObject : INotifyPropertyChanged
{
    public event PropertyChangedEventHandler? PropertyChanged;

    protected bool SetProperty<T>(ref T field, T value, [CallerMemberName] string? propertyName = null)
    {
        if (EqualityComparer<T>.Default.Equals(field, value))
        {
            return false;
        }
        field = value;
        OnPropertyChanged(propertyName);
        return true;
    }

    protected void OnPropertyChanged([CallerMemberName] string? propertyName = null)
        => PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
}
