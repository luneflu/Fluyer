import SwiftUI
import FluyerCore

/// Right-hand queue sidebar. Port of `MusicQueueList.svelte` inside `Sidebar.svelte`:
/// glass card two album slots wide (width supplied by the caller), "Now Playing"
/// header with a clear button, grid-sized rows whose hover overlay turns the cover
/// into a jump button and swaps the duration for remove. Rows drag to reorder.
struct QueuePaneView: View {
    @Bindable var state: AppState
    let width: CGFloat

    var body: some View {
        let tracks = state.queue.tracks

        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 12) {
                Text("Now Playing")
                    .font(.system(size: 22, weight: .semibold))
                    .lineLimit(1)
                Spacer(minLength: 0)
                Button { state.queue.clear() } label: {
                    Image(systemName: "trash")
                        .font(.system(size: 16))
                        .frame(width: 28, height: 28)
                }
                .buttonStyle(.plain)
                .disabled(tracks.isEmpty)
                .help("Clear Queue")
                .accessibilityLabel("Clear Queue")
            }
            .padding(12)

            if tracks.isEmpty {
                Text("Queue is empty")
                    .font(.system(size: 13))
                    .foregroundColor(.white.opacity(0.5))
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollViewReader { proxy in
                    // `List` for native drag-to-reorder: drop indicator, edge
                    // auto-scroll and accessibility, with no custom drag code.
                    List {
                        let current = tracks.firstIndex(where: \.isCurrent)
                        // Position ids: the same track can be queued twice.
                        ForEach(Array(tracks.enumerated()), id: \.offset) { row, track in
                            QueueRow(state: state, track: track, row: row, count: tracks.count, current: current)
                                .id(row)
                                .listRowInsets(EdgeInsets())
                                .listRowSeparator(.hidden)
                                .listRowBackground(Color.clear)
                        }
                        .onMove { source, destination in
                            guard let from = source.first else { return }
                            state.queue.move(from: from, toOffset: destination)
                        }
                    }
                    .listStyle(.plain)
                    .scrollContentBackground(.hidden)
                    .scrollIndicators(.hidden)
                    .environment(\.defaultMinListRowHeight, 1)
                    .onAppear {
                        if let current = tracks.firstIndex(where: \.isCurrent) {
                            proxy.scrollTo(current, anchor: .center)
                        }
                    }
                }
            }
        }
        .padding(12)
        .foregroundColor(.white)
        .frame(width: width)
        .frame(maxHeight: .infinity)
        // `View.svelte`: bg-gray-300/10 glass, `rounded`.
        .background(.ultraThinMaterial.opacity(0.9))
        .background(Color(white: 0.83).opacity(0.10))
        .clipShape(RoundedRectangle(cornerRadius: 4))
        .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.white.opacity(0.12), lineWidth: 1))
    }
}

private struct QueueRow: View {
    let state: AppState
    let track: TrackItemViewModel
    let row: Int
    let count: Int
    let current: Int?

    @State private var isHovering = false
    @State private var cover: NSImage?

    /// Same geometry and cache key as `MusicGridView.TrackCard`, so covers are shared.
    private static let side: CGFloat = 44
    private static let pixels = 88

    /// Queue rows carry the queue position in `index`; covers are keyed by library
    /// index, so resolve it by path.
    private var libraryIndex: UInt64? {
        state.library.tracks.first(where: { $0.path == track.path })?.index
    }

    var body: some View {
        HStack(spacing: 10) {
            coverView
            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    if track.isCurrent {
                        Image(systemName: state.playback.bar.isPlaying ? "speaker.wave.2.fill" : "speaker.fill")
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
            Spacer(minLength: 4)
            trailing
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .contentShape(Rectangle())
        .onHover { isHovering = $0 }
        .contextMenu {
            Button("Play") { state.queue.goto(row) }
            Button("Move Up") { state.queue.move(row, by: -1) }.disabled(row == 0)
            Button("Move Down") { state.queue.move(row, by: 1) }.disabled(row >= count - 1)
            Button("Remove") { state.queue.remove(row) }
        }
        .task(id: libraryIndex) { await loadCover() }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(track.isCurrent ? "Now playing: \(track.title), \(track.artist)" : "\(track.title), \(track.artist)")
        .accessibilityAddTraits(.isButton)
        .accessibilityAction { state.queue.goto(row) }
    }

    private var coverView: some View {
        ZStack {
            if let cover {
                Image(nsImage: cover).resizable().aspectRatio(contentMode: .fill)
            } else {
                Color.white.opacity(0.08)
            }
            if isHovering && !track.isCurrent {
                Button { state.queue.goto(row) } label: {
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
        .clipShape(RoundedRectangle(cornerRadius: 4))
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
                Button { state.queue.remove(row) } label: {
                    Image(systemName: "xmark").font(.system(size: 12))
                        .frame(width: 24, height: 24)
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
        guard let engine = state.engine, let index = libraryIndex else { return }
        let key = ThumbnailKey.track(index, px: Self.pixels)
        if let hit = ThumbnailStore.shared.peek(key) {
            cover = hit
            return
        }
        cover = await ThumbnailStore.shared.image(key: key) {
            await engine.loadTrackThumbnail(index: index, maxSize: UInt32(Self.pixels))
        }
    }
}
