import SwiftUI
import FluyerCore

/// Right-hand queue sidebar. Port of `MusicQueueList.svelte` inside `Sidebar.svelte`:
/// glass card two album slots wide (width supplied by the caller), "Now Playing"
/// header with a clear button, grid-sized rows whose hover overlay turns the cover
/// into a jump button and swaps the duration for remove. Rows drag to reorder.
struct QueueView: View {
    @Bindable var queue: QueueState
    let playback: PlaybackState
    let library: LibraryState
    let covers: CoverState
    let width: CGFloat

    var body: some View {
        let tracks = queue.tracks

        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: Layout.Queue.headerSpacing) {
                Text("Now Playing")
                    .font(.system(size: 22, weight: .semibold))
                    .lineLimit(1)
                Spacer(minLength: 0)
                Button { queue.clear() } label: {
                    Image(systemName: "trash")
                        .font(.system(size: 16))
                        .frame(width: Layout.Queue.clearButtonSide, height: Layout.Queue.clearButtonSide)
                }
                .buttonStyle(.plain)
                .disabled(tracks.isEmpty)
                .help("Clear Queue")
                .accessibilityLabel("Clear Queue")
            }
            .padding(Layout.Queue.headerPadding)

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
                            QueueRow(queue: queue, playback: playback, library: library, covers: covers, track: track, row: row, count: tracks.count, current: current)
                                .id(row)
                                .listRowInsets(EdgeInsets())
                                .listRowSeparator(.hidden)
                                .listRowBackground(Color.clear)
                        }
                        .onMove { source, destination in
                            guard let from = source.first else { return }
                            queue.move(from: from, toOffset: destination)
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
        .padding(Layout.Queue.panelPadding)
        .foregroundColor(.white)
        .frame(width: width)
        .frame(maxHeight: .infinity)
        // `View.svelte`: bg-gray-300/10 glass, `rounded`.
        .background(.ultraThinMaterial.opacity(0.9))
        .background(Color(white: 0.83).opacity(0.10))
        .clipShape(RoundedRectangle(cornerRadius: Layout.Queue.cornerRadius))
        .overlay(RoundedRectangle(cornerRadius: Layout.Queue.cornerRadius).stroke(Color.white.opacity(0.12), lineWidth: 1))
    }
}
