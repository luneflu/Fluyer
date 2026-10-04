using Microsoft.UI.Xaml;
using Windows.Storage.Pickers;
using WinRT.Interop;

namespace Fluyer.Support;

/// <summary>
/// WinUI port of <c>FolderPicker</c> (<c>ui/macos/Sources/Support/FolderPicker.swift</c>).
/// WinRT's folder picker is single-folder per invocation
/// (<c>PickSingleFolderAsync</c>), unlike <c>NSOpenPanel</c> — multi-folder
/// comes from repeated picks.
/// </summary>
public static class FolderPicker
{
    public static async Task<string[]> PickMusicFoldersAsync(Window window)
    {
        var picker = new Windows.Storage.Pickers.FolderPicker
        {
            SuggestedStartLocation = PickerLocationId.MusicLibrary,
        };
        picker.FileTypeFilter.Add("*");
        InitializeWithWindow.Initialize(picker, WindowNative.GetWindowHandle(window));

        var folder = await picker.PickSingleFolderAsync();
        return folder is null ? [] : [folder.Path];
    }
}
