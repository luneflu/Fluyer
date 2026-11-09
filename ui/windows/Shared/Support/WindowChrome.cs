using System.Runtime.InteropServices;
using Microsoft.UI.Xaml;
using WinRT.Interop;

namespace Fluyer.Shared;

/// <summary>
/// DWM frame colors live outside the XAML content. WinUI's
/// <c>ExtendsContentIntoTitleBar</c> does not touch them, so a dark app can
/// still get a light 1px border along the window edge when restored.
/// </summary>
public static class WindowChrome
{
    private const int UseImmersiveDarkMode = 20;
    private const int UseImmersiveDarkModePre20H1 = 19;
    private const int BorderColor = 34;
    private const int ColorNone = unchecked((int)0xFFFFFFFE);

    public static void ApplyDarkFrame(Window window)
    {
        var hwnd = WindowNative.GetWindowHandle(window);
        var dark = 1;
        if (DwmSetWindowAttribute(hwnd, UseImmersiveDarkMode, ref dark, sizeof(int)) != 0)
        {
            // Windows 10 1809-1909 use attribute 19.
            _ = DwmSetWindowAttribute(hwnd, UseImmersiveDarkModePre20H1, ref dark, sizeof(int));
        }

        var none = ColorNone;
        _ = DwmSetWindowAttribute(hwnd, BorderColor, ref none, sizeof(int));
    }

    [DllImport("dwmapi.dll")]
    private static extern int DwmSetWindowAttribute(IntPtr hwnd, int attribute, ref int value, int size);
}
