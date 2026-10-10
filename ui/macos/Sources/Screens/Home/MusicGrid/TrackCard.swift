import SwiftUI
import FluyerCore

struct TrackCard: View {
    let filter: LibraryFilterState
    let playback: PlaybackState
    let covers: CoverState
    let track: TrackItemViewModel

    @State private var thumbnail: NSImage?

    private var isCurrent: Bool { track.isCurrent }

    /// Inside an album every row renders the album cover, so they share one cache
    /// entry; otherwise each row uses its own track cover.
    private var cacheKey: String {
        let pixels = Self.pixels
        if let albumIndex = filter.index {
            return ThumbnailKey.album(UInt64(albumIndex), px: pixels)
        }
        return ThumbnailKey.track(track.index, px: pixels)
    }

    private static let pixels = Layout.TrackRow.coverPixels

    var body: some View {
        HStack(spacing: Layout.TrackRow.coverToText) {
            thumbnailView

            VStack(alignment: .leading, spacing: Layout.TrackRow.titleToSubtitle) {
                HStack(spacing: Layout.TrackRow.iconToTitle) {
                    if isCurrent {
                        Image(systemName: playback.bar.isPlaying ? "speaker.wave.2.fill" : "speaker.fill")
                            .font(.system(size: 10))
                            .foregroundStyle(.tint)
                    }

                    Text(track.title)
                        .font(.system(size: 13, weight: isCurrent ? .semibold : .regular))
                        .foregroundStyle(isCurrent ? AnyShapeStyle(.tint) : AnyShapeStyle(.white))
                        .lineLimit(1)
                }

                Text("\(track.artist) • \(track.album)")
                    .font(.system(size: 11))
                    .foregroundColor(.white.opacity(0.6))
                    .lineLimit(1)
            }

            Spacer(minLength: Layout.TrackRow.minTextToTrailing)

            Text(track.durationFormatted)
                .font(.system(size: 11, design: .monospaced))
                .foregroundColor(.white.opacity(0.4))
        }
        .padding(.vertical, Layout.TrackRow.verticalPadding)
        .contentShape(Rectangle())
        .onTapGesture {
            filter.playTrack(track)
        }
        // ponytail: .task cancels automatically when the row scrolls out of the
        // lazy stack, so fast scrolling never queues unbounded decode work.
        .task(id: cacheKey) {
            await loadThumbnail()
        }
    }

    @ViewBuilder
    private var thumbnailView: some View {
        ZStack {
            if let thumbnail {
                Image(nsImage: thumbnail)
                    .resizable()
                    .aspectRatio(contentMode: .fill)
            } else {
                Rectangle()
                    .fill(Color.white.opacity(0.08))
                    .overlay(
                        Image(systemName: "music.note")
                            .font(.system(size: 16))
                            .foregroundColor(.white.opacity(0.3))
                    )
            }
        }
        .frame(width: Layout.TrackRow.coverSide, height: Layout.TrackRow.coverSide)
    }

    private func loadThumbnail() async {
        // ponytail: no peek fast-path here — `CoverState` (via `ThumbnailStore.image`) peeks first and
        // returns before awaiting the core, so the extra call was pure duplication.
        if let albumIndex = filter.index {
            thumbnail = await covers.album(UInt64(albumIndex), px: Self.pixels)
        } else {
            thumbnail = await covers.track(track.index, px: Self.pixels)
        }
    }
}
