using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.WindowsRuntime;
using Fluyer.Core.Native;
using Fluyer.Core.State;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Windows.Media;
using Windows.Storage.Streams;
using WinRT;
using WinRT.Interop;

namespace Fluyer.Shared;

/// <summary>
/// Bridges Windows System Media Transport Controls (SMTC) to <see cref="PlaybackState"/>.
/// Controls media keys (Play/Pause, Next, Prev), taskbar hover controls, volume flyout,
/// lock screen now-playing, and timeline scrubber.
/// </summary>
public sealed class MediaTransportCoordinator : IDisposable
{
    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    private delegate int GetForWindowDelegate(
        IntPtr thisPtr,
        IntPtr appWindow,
        [In] ref Guid riid,
        out IntPtr mediaTransportControl);

    [DllImport("combase.dll", CharSet = CharSet.Unicode, ExactSpelling = true)]
    private static extern int WindowsCreateString(
        string sourceString,
        int length,
        out IntPtr hstring);

    [DllImport("combase.dll", ExactSpelling = true)]
    private static extern int WindowsDeleteString(IntPtr hstring);

    [DllImport("combase.dll", PreserveSig = false)]
    private static extern void RoGetActivationFactory(
        IntPtr activatableClassId,
        [In] ref Guid iid,
        out IntPtr factory);

    private static SystemMediaTransportControls? GetForWindow(IntPtr hwnd)
    {
        const string className = "Windows.Media.SystemMediaTransportControls";
        if (WindowsCreateString(className, className.Length, out var hstring) != 0 || hstring == IntPtr.Zero)
        {
            return null;
        }

        try
        {
            // ISystemMediaTransportControlsInterop IID: ddb0472d-c911-4a1f-86d9-dc3d71a95f5a
            var interopIid = new Guid("ddb0472d-c911-4a1f-86d9-dc3d71a95f5a");
            RoGetActivationFactory(hstring, ref interopIid, out var factoryPtr);
            if (factoryPtr == IntPtr.Zero)
            {
                return null;
            }

            try
            {
                // Slot 6 = GetForWindow (0..2 IUnknown, 3..5 IInspectable, 6 GetForWindow)
                var vtable = Marshal.ReadIntPtr(factoryPtr);
                var methodPtr = Marshal.ReadIntPtr(vtable, 6 * IntPtr.Size);
                var getForWindow = Marshal.GetDelegateForFunctionPointer<GetForWindowDelegate>(methodPtr);

                // ISystemMediaTransportControls IID: 99FA3FF4-1742-42A6-902E-087D41F965EC
                var smtcIid = new Guid("99FA3FF4-1742-42A6-902E-087D41F965EC");
                if (getForWindow(factoryPtr, hwnd, ref smtcIid, out var smtcPtr) != 0 || smtcPtr == IntPtr.Zero)
                {
                    return null;
                }

                return MarshalInspectable<SystemMediaTransportControls>.FromAbi(smtcPtr);
            }
            finally
            {
                Marshal.Release(factoryPtr);
            }
        }
        catch
        {
            return null;
        }
        finally
        {
            WindowsDeleteString(hstring);
        }
    }

    private readonly PlaybackState _playback;
    private readonly CoverState _covers;
    private readonly SystemMediaTransportControls? _smtc;
    private readonly DispatcherQueue _dispatcherQueue;
    private string _currentTrackKey = string.Empty;
    private bool _disposed;

    public MediaTransportCoordinator(Window window, PlaybackState playback, CoverState covers)
    {
        _playback = playback;
        _covers = covers;
        _dispatcherQueue = window.DispatcherQueue;

        var hwnd = WindowNative.GetWindowHandle(window);
        _smtc = GetForWindow(hwnd);
        if (_smtc is null)
        {
            return;
        }

        _smtc.IsEnabled = true;
        _smtc.IsPlayEnabled = true;
        _smtc.IsPauseEnabled = true;
        _smtc.IsNextEnabled = true;
        _smtc.IsPreviousEnabled = true;
        _smtc.IsStopEnabled = true;

        _smtc.ButtonPressed += OnButtonPressed;
        _smtc.PlaybackPositionChangeRequested += OnPlaybackPositionChangeRequested;
        _smtc.AutoRepeatModeChangeRequested += OnAutoRepeatModeChangeRequested;
        _smtc.ShuffleEnabledChangeRequested += OnShuffleEnabledChangeRequested;

        _playback.PropertyChanged += OnPlaybackPropertyChanged;

        UpdateTransport();
    }

    private void OnButtonPressed(SystemMediaTransportControls sender, SystemMediaTransportControlsButtonPressedEventArgs args)
    {
        _dispatcherQueue.TryEnqueue(() =>
        {
            switch (args.Button)
            {
                case SystemMediaTransportControlsButton.Play:
                    _playback.Play();
                    break;
                case SystemMediaTransportControlsButton.Pause:
                case SystemMediaTransportControlsButton.Stop:
                    _playback.Pause();
                    break;
                case SystemMediaTransportControlsButton.Next:
                    _playback.Next();
                    break;
                case SystemMediaTransportControlsButton.Previous:
                    _playback.Previous();
                    break;
            }
        });
    }

    private void OnPlaybackPositionChangeRequested(
        SystemMediaTransportControls sender,
        PlaybackPositionChangeRequestedEventArgs args)
    {
        var targetMs = (ulong)Math.Max(0, args.RequestedPlaybackPosition.TotalMilliseconds);
        _dispatcherQueue.TryEnqueue(() => _playback.Seek(targetMs));
    }

    private void OnAutoRepeatModeChangeRequested(
        SystemMediaTransportControls sender,
        AutoRepeatModeChangeRequestedEventArgs args)
    {
        _dispatcherQueue.TryEnqueue(() => _playback.CycleRepeat());
    }

    private void OnShuffleEnabledChangeRequested(
        SystemMediaTransportControls sender,
        ShuffleEnabledChangeRequestedEventArgs args)
    {
        _dispatcherQueue.TryEnqueue(() => _playback.Shuffle());
    }

    private void OnPlaybackPropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(PlaybackState.Bar))
        {
            UpdateTransport();
        }
    }

    private void UpdateTransport()
    {
        if (_smtc is null || _disposed)
        {
            return;
        }

        var bar = _playback.Bar;
        var hasTrack = bar.Title != "No Track" && !string.IsNullOrEmpty(bar.Title);

        _smtc.PlaybackStatus = bar.IsPlaying
            ? MediaPlaybackStatus.Playing
            : (hasTrack ? MediaPlaybackStatus.Paused : MediaPlaybackStatus.Stopped);

        _smtc.AutoRepeatMode = bar.RepeatMode switch
        {
            RepeatMode.All => MediaPlaybackAutoRepeatMode.List,
            RepeatMode.One => MediaPlaybackAutoRepeatMode.Track,
            _ => MediaPlaybackAutoRepeatMode.None,
        };
        _smtc.ShuffleEnabled = bar.IsShuffled;

        var timeline = new SystemMediaTransportControlsTimelineProperties
        {
            StartTime = TimeSpan.Zero,
            EndTime = TimeSpan.FromMilliseconds(bar.DurationMs),
            Position = TimeSpan.FromMilliseconds(bar.PositionMs),
        };
        _smtc.UpdateTimelineProperties(timeline);

        var trackKey = $"{bar.TrackIndex}|{bar.Title}|{bar.Artist}|{bar.Album}";
        if (trackKey != _currentTrackKey)
        {
            _currentTrackKey = trackKey;
            _ = UpdateMetadataAndThumbnailAsync(bar, trackKey);
        }
    }

    private async Task UpdateMetadataAndThumbnailAsync(PlayerBarViewModel bar, string trackKey)
    {
        if (_smtc is null || _disposed)
        {
            return;
        }

        var updater = _smtc.DisplayUpdater;
        updater.Type = MediaPlaybackType.Music;

        if (bar.Title == "No Track" || string.IsNullOrEmpty(bar.Title))
        {
            updater.ClearAll();
            updater.Update();
            return;
        }

        updater.MusicProperties.Title = bar.Title;
        updater.MusicProperties.Artist = bar.Artist;
        updater.MusicProperties.AlbumTitle = bar.Album;

        var thumb = await _covers.CurrentBytes(400).ConfigureAwait(true);

        if (_currentTrackKey != trackKey || _disposed)
        {
            return;
        }

        if (thumb is { Length: > 0 })
        {
            var stream = new InMemoryRandomAccessStream();
            await stream.WriteAsync(thumb.AsBuffer()).AsTask().ConfigureAwait(true);
            stream.Seek(0);
            updater.Thumbnail = RandomAccessStreamReference.CreateFromStream(stream);
        }
        else
        {
            updater.Thumbnail = null;
        }

        updater.Update();
    }

    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }
        _disposed = true;

        _playback.PropertyChanged -= OnPlaybackPropertyChanged;

        if (_smtc is not null)
        {
            _smtc.ButtonPressed -= OnButtonPressed;
            _smtc.PlaybackPositionChangeRequested -= OnPlaybackPositionChangeRequested;
            _smtc.AutoRepeatModeChangeRequested -= OnAutoRepeatModeChangeRequested;
            _smtc.ShuffleEnabledChangeRequested -= OnShuffleEnabledChangeRequested;

            try
            {
                _smtc.PlaybackStatus = MediaPlaybackStatus.Closed;
                _smtc.IsEnabled = false;
                _smtc.DisplayUpdater.ClearAll();
                _smtc.DisplayUpdater.Update();
            }
            catch
            {
                // Process shutdown cleanup
            }
        }
    }
}
