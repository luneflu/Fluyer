using Microsoft.UI.Xaml;

namespace Fluyer.Shared;

/// <summary>
/// Code-behind access to <c>Shared/Theme/Layout.xaml</c>, so layout math in C#
/// reads the same numbers XAML uses instead of keeping its own copies.
/// Keys are <c>&lt;Feature&gt;&lt;Role&gt;</c>, e.g. <c>"PlayColumnGutter"</c>.
/// </summary>
public static class Layout
{
    public static double Number(string key) => (double)Application.Current.Resources[key];

    public static Thickness Edges(string key) => (Thickness)Application.Current.Resources[key];
}
