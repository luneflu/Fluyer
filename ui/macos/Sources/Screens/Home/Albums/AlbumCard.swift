import SwiftUI
import FluyerCore

/// One album cover with its labels (`AlbumItem.svelte`). Click opens the album,
/// right-click plays / queues / shuffles it. While another album is open, the rest
/// dim until the pointer enters the list.
///
/// The card itself is the shared `CoverCard`; this wrapper only maps local library
/// state (thumbnail cache key, selection, album actions) onto it.
struct AlbumCard: View {
    let filter: LibraryFilterState
    let covers: CoverState
    let album: AlbumCardViewModel
    let coverSize: CGFloat
    var isListHovered = false

    private var isSelected: Bool { filter.index == Int(album.index) }

    private var isDimmed: Bool { filter.isActive && !isSelected && !isListHovered }

    private var actions: [(title: String, run: () -> Void)] {
        [
            ("Play", { filter.playAlbum(Int(album.index)) }),
            ("Add to Queue", { filter.queueAlbum(Int(album.index)) }),
            ("Shuffle", { filter.shuffleAlbum(Int(album.index)) }),
        ]
    }

    var body: some View {
        CoverCard(
            coverSize: coverSize,
            title: album.name,
            subtitle: album.artist,
            isSelected: isSelected,
            isDimmed: isDimmed,
            actions: actions,
            onTap: { filter.select(Int(album.index)) },
            cached: { covers.cached(ThumbnailKey.album(album.index, px: $0)) },
            load: { await covers.album(album.index, px: $0) }
        )
    }
}
