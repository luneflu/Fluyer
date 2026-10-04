using System.Collections.ObjectModel;
using Fluyer.Core.Native;
using Fluyer.Core.Support;

namespace Fluyer.Core.State;

/// <summary>
/// The scanned library: flat track list, album list and scan progress.
/// Port of <c>LibraryState</c> (<c>ui/macos/Sources/State/LibraryState.swift</c>).
/// </summary>
public sealed class LibraryState : Support.ObservableObject
{
    private ScanStatusViewModel _scanStatus = ViewModelDefaults.Idle;

    public ObservableCollection<TrackItemViewModel> Tracks { get; } = new();
    public ObservableCollection<AlbumCardViewModel> Albums { get; } = new();

    public ScanStatusViewModel ScanStatus
    {
        get => _scanStatus;
        set => SetProperty(ref _scanStatus, value);
    }

    public IFluyerEngine? Engine { get; set; }

    /// <summary>Re-read the whole library from the core.</summary>
    public void Reload()
    {
        if (Engine is null)
        {
            return;
        }
        ScanStatus = ViewModelDefaults.Idle;
        var tracks = new List<TrackItemViewModel>();
        var trackCount = Engine.GetTrackCount();
        for (ulong i = 0; i < trackCount; i++)
        {
            var view = Engine.GetTrackView(i);
            if (view is not null)
            {
                tracks.Add(view);
            }
        }
        var albums = new List<AlbumCardViewModel>();
        var albumCount = Engine.GetAlbumCount();
        for (ulong i = 0; i < albumCount; i++)
        {
            var card = Engine.GetAlbumCard(i);
            if (card is not null)
            {
                albums.Add(card);
            }
        }
        ReplaceAll(Tracks, tracks);
        ReplaceAll(Albums, albums);
    }

    /// <summary>
    /// Refresh only the "now playing" marker, so a track change does not
    /// rebuild every row in the grid.
    /// </summary>
    public void ReloadActiveFlags()
    {
        if (Engine is null || Tracks.Count != checked((int)Engine.GetTrackCount()))
        {
            return;
        }
        for (var row = 0; row < Tracks.Count; row++)
        {
            var updated = Engine.GetTrackView((ulong)row);
            if (updated is not null && updated.IsCurrent != Tracks[row].IsCurrent)
            {
                Tracks[row] = Tracks[row] with { IsCurrent = updated.IsCurrent };
            }
        }
    }

    private static void ReplaceAll<T>(ObservableCollection<T> collection, List<T> items)
    {
        collection.Clear();
        foreach (var item in items)
        {
            collection.Add(item);
        }
    }
}
