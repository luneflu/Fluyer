import SwiftUI
import FluyerCore

/// One album cover with its labels (`AlbumItem.svelte`). Click opens the album,
/// right-click plays / queues / shuffles it. While another album is open, the rest
/// dim until the pointer enters the list.
struct AlbumCard: View {
    let filter: LibraryFilterState
    let covers: CoverState
    let album: AlbumCardViewModel
    let coverSize: CGFloat
    var isListHovered = false

    @State private var thumbnail: NSImage?
    @State private var isHovered = false

    /// Requested at 2x for Retina, which is also what makes this key distinct from
    /// the 88px album thumbnail the track rows ask for.
    private var pixelSize: Int { max(16, Int(coverSize * 2)) }

    private var cacheKey: String { ThumbnailKey.album(album.index, px: pixelSize) }

    private var isSelected: Bool { filter.index == Int(album.index) }

    private var isDimmed: Bool { filter.isActive && !isSelected && !isListHovered }

    var body: some View {
        VStack(alignment: .leading, spacing: Layout.Albums.coverToLabels) {
            cover
            labels
        }
        .frame(width: coverSize)
        .opacity(isDimmed ? 0.4 : 1)
        .animation(.easeInOut(duration: 0.3), value: isDimmed)
        .contentShape(Rectangle())
        .onHover { isHovered = $0 }
        .onTapGesture {
            filter.select(Int(album.index))
        }
        .contextMenu {
            Button("Play") { filter.playAlbum(Int(album.index)) }
            Button("Add to Queue") { filter.queueAlbum(Int(album.index)) }
            Button("Shuffle") { filter.shuffleAlbum(Int(album.index)) }
        }
        .accessibilityElement(children: .combine)
        .accessibilityAddTraits(.isButton)
        .accessibilityAddTraits(isSelected ? .isSelected : [])
        .task(id: cacheKey) {
            // Warm cache: show at once. Cold: fade in once decoded (`anim-fade-in`).
            if let hit = covers.cached(cacheKey) {
                thumbnail = hit
                return
            }
            guard let image = await covers.album(album.index, px: pixelSize) else { return }
            withAnimation(.easeIn(duration: 0.3)) { thumbnail = image }
        }
    }

    @ViewBuilder
    private var cover: some View {
        ZStack {
            Rectangle()
                .fill(Color.white.opacity(0.08))
                .overlay(
                    Image(systemName: "music.note")
                        .font(.system(size: coverSize * 0.25))
                        .foregroundColor(.white.opacity(0.3))
                )
            if let thumbnail {
                Image(nsImage: thumbnail)
                    .resizable()
                    .aspectRatio(1, contentMode: .fill)
                    .transition(.opacity)
            }
            // Selected: solid border. Others: border + white wash on hover.
            RoundedRectangle(cornerRadius: Layout.Albums.coverCornerRadius)
                .fill(Color.white.opacity(isSelected ? 0 : 0.2))
                .strokeBorder(Color.white, lineWidth: 2)
                .opacity(isSelected || isHovered ? 1 : 0)
                .animation(.easeInOut(duration: isHovered ? 0.5 : 0.75), value: isHovered)
        }
        .frame(width: coverSize, height: coverSize)
        .clipShape(RoundedRectangle(cornerRadius: Layout.Albums.coverCornerRadius))
        .shadow(color: .black.opacity(0.25), radius: 6, x: 0, y: 3)
    }

    private var labels: some View {
        VStack(alignment: .leading, spacing: Layout.Albums.nameToArtist) {
            Text(album.name)
                .font(.system(size: 13, weight: isSelected ? .semibold : .medium))
                .foregroundColor(.white)
                .lineLimit(1)

            Text(album.artist)
                .font(.system(size: 11))
                .foregroundColor(.white.opacity(0.6))
                .lineLimit(1)
        }
        .frame(width: coverSize, alignment: .leading)
    }
}
