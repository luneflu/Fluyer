import SwiftUI
import FluyerCore

/// One remote track, styled like the library's `TrackCard` (`Layout.TrackRow` metrics,
/// no hover chrome) so search results and feed shelves match the song grid. Only
/// playable when the item carries a video id.
struct YoutubeTrackRow: View {
    let item: SearchItem
    let isPlaying: Bool
    let onPlay: () -> Void

    var body: some View {
        HStack(spacing: Layout.TrackRow.coverToText) {
            thumbnailView

            VStack(alignment: .leading, spacing: Layout.TrackRow.titleToSubtitle) {
                HStack(spacing: Layout.TrackRow.iconToTitle) {
                    if isPlaying {
                        Image(systemName: "speaker.wave.2.fill")
                            .font(.system(size: 10))
                            .foregroundStyle(.tint)
                    }

                    Text(item.title)
                        .font(.system(size: 13, weight: isPlaying ? .semibold : .regular))
                        .foregroundStyle(isPlaying ? AnyShapeStyle(.tint) : AnyShapeStyle(.white))
                        .lineLimit(1)
                }

                Text(item.subtitle ?? item.kind.label)
                    .font(.system(size: 11))
                    .foregroundColor(.white.opacity(0.6))
                    .lineLimit(1)
            }

            Spacer(minLength: Layout.TrackRow.minTextToTrailing)

            // No duration in the feed payload, so only non-track kinds label themselves.
            if item.videoId == nil {
                Text(item.kind.label.uppercased())
                    .font(.system(size: 9, weight: .semibold))
                    .foregroundColor(.white.opacity(0.35))
            }
        }
        .padding(.vertical, Layout.TrackRow.verticalPadding)
        .contentShape(Rectangle())
        .onTapGesture {
            guard item.videoId != nil else { return }
            onPlay()
        }
    }

    private var thumbnailView: some View {
        AsyncImage(url: item.thumbnail.flatMap(URL.init(string:))) { image in
            image.resizable().aspectRatio(contentMode: .fill)
        } placeholder: {
            Rectangle()
                .fill(Color.white.opacity(0.08))
                .overlay(
                    Image(systemName: "music.note")
                        .font(.system(size: 16))
                        .foregroundColor(.white.opacity(0.3))
                )
        }
        .frame(width: Layout.TrackRow.coverSide, height: Layout.TrackRow.coverSide)
    }
}

extension SearchItemKind {
    var label: String {
        switch self {
        case .track: "Song"
        case .album: "Album"
        case .artist: "Artist"
        case .playlist: "Playlist"
        }
    }
}
