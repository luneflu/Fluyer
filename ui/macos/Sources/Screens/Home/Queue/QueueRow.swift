import SwiftUI
import FluyerCore

struct QueueRow: View {
    let queue: QueueState
    let playback: PlaybackState
    let library: LibraryState
    let covers: CoverState
    let track: TrackItemViewModel
    let row: Int
    let count: Int
    let current: Int?

    @State private var isHovering = false
    @State private var cover: NSImage?

    /// Same geometry and cache key as `TrackCard` (`Layout.TrackRow`), so covers are shared.
    private static let side = Layout.TrackRow.coverSide
    private static let pixels = Layout.TrackRow.coverPixels

    /// Queue rows carry the queue position in `index`; covers are keyed by library
    /// index, so resolve it by path.
    private var libraryIndex: UInt64? {
        library.tracks.first(where: { $0.path == track.path })?.index
    }

    var body: some View {
        HStack(spacing: Layout.TrackRow.coverToText) {
            coverView
            VStack(alignment: .leading, spacing: Layout.TrackRow.titleToSubtitle) {
                HStack(spacing: Layout.TrackRow.iconToTitle) {
                    if track.isCurrent {
                        Image(systemName: playback.bar.isPlaying ? "speaker.wave.2.fill" : "speaker.fill")
                            .font(.system(size: 10))
                            .foregroundStyle(.tint)
                            .accessibilityLabel("Now playing")
                    }
                    Text(track.title)
                        .font(.system(size: 13, weight: track.isCurrent ? .semibold : .regular))
                        .foregroundStyle(track.isCurrent ? AnyShapeStyle(.tint) : AnyShapeStyle(.white))
                        .lineLimit(1)
                }
                Text("\(track.artist) • \(track.album)")
                    .font(.system(size: 11))
                    .foregroundColor(.white.opacity(0.6))
                    .lineLimit(1)
            }
            Spacer(minLength: Layout.TrackRow.minTextToTrailing)
            trailing
        }
        .padding(.horizontal, Layout.Queue.rowHorizontalPadding)
        .padding(.vertical, Layout.TrackRow.verticalPadding)
        .contentShape(Rectangle())
        .onHover { isHovering = $0 }
        .contextMenu {
            Button("Play") { queue.goto(row) }
            Button("Move Up") { queue.move(row, by: -1) }.disabled(row == 0)
            Button("Move Down") { queue.move(row, by: 1) }.disabled(row >= count - 1)
            Button("Remove") { queue.remove(row) }
        }
        .task(id: libraryIndex) { await loadCover() }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(track.isCurrent ? "Now playing: \(track.title), \(track.artist)" : "\(track.title), \(track.artist)")
        .accessibilityAddTraits(.isButton)
        .accessibilityAction { queue.goto(row) }
    }

    private var coverView: some View {
        ZStack {
            if let cover {
                Image(nsImage: cover).resizable().aspectRatio(contentMode: .fill)
            } else {
                Color.white.opacity(0.08)
            }
            if isHovering && !track.isCurrent {
                Button { queue.goto(row) } label: {
                    ZStack {
                        Color.black.opacity(0.4)
                        Image(systemName: current.map { row < $0 } == true ? "backward.fill" : "forward.fill")
                            .font(.system(size: 14))
                    }
                }
                .buttonStyle(.plain)
                .help("Play")
                .transition(.opacity)
            }
        }
        .frame(width: Self.side, height: Self.side)
        .clipShape(RoundedRectangle(cornerRadius: Layout.TrackRow.coverCornerRadius))
    }

    /// Duration like the grid; on hover a non-playing row swaps it for remove.
    @ViewBuilder
    private var trailing: some View {
        ZStack(alignment: .trailing) {
            Text(track.durationFormatted)
                .font(.system(size: 11, design: .monospaced))
                .foregroundColor(.white.opacity(0.4))
                .opacity(isHovering && !track.isCurrent ? 0 : 1)
            if isHovering && !track.isCurrent {
                Button { queue.remove(row) } label: {
                    Image(systemName: "xmark").font(.system(size: 12))
                        .frame(width: Layout.Queue.removeButtonSide, height: Layout.Queue.removeButtonSide)
                }
                .buttonStyle(.plain)
                .help("Remove")
                .accessibilityLabel("Remove")
                .transition(.opacity)
            }
        }
        .animation(.easeInOut(duration: 0.2), value: isHovering)
    }

    private func loadCover() async {
        guard let index = libraryIndex else { return }
        if let hit = covers.cached(ThumbnailKey.track(index, px: Self.pixels)) {
            cover = hit
            return
        }
        cover = await covers.track(index, px: Self.pixels)
    }
}
