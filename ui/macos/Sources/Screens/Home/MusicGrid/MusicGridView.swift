import SwiftUI
import FluyerCore

/// Adaptive grid of track rows. Shows the whole library, or one album's tracks when
/// `LibraryFilterState` is active.
struct MusicGridView: View {
    let filter: LibraryFilterState
    let library: LibraryState
    let playback: PlaybackState
    let covers: CoverState
    /// "Open Music Folder..." in the empty state.
    let onAddFolder: () -> Void

    private let columns = [
        GridItem(.adaptive(minimum: Layout.MusicGrid.minColumnWidth, maximum: .infinity),
                 spacing: Layout.MusicGrid.columnSpacing)
    ]

    var body: some View {
        let tracks = filter.displayedTracks

        Group {
            if tracks.isEmpty {
                emptyState
            } else {
                ScrollView {
                    LazyVGrid(columns: columns, spacing: Layout.MusicGrid.rowSpacing) {
                        ForEach(tracks, id: \.path) { track in
                            TrackCard(filter: filter, playback: playback, covers: covers, track: track)
                                .hiddenBySidebar()
                        }
                    }
                    .padding(.horizontal, Layout.MusicGrid.horizontalPadding)
                    .padding(.vertical, Layout.MusicGrid.verticalPadding)
                }
                // Scroll bar would sit on the right-edge queue trigger (`scrollbar-hidden`).
                .scrollIndicators(.hidden)
            }
        }
    }

    private var emptyState: some View {
        VStack(spacing: Layout.MusicGrid.emptyStateSpacing) {
            Spacer()
            Image(systemName: "music.note.list")
                .font(.system(size: 36))
                .foregroundColor(.white.opacity(0.2))
            Text(library.tracks.isEmpty ? "No songs in library" : "No songs found")
                .font(.system(size: 14))
                .foregroundColor(.white.opacity(0.5))
            if library.tracks.isEmpty {
                Button("Open Music Folder...") {
                    onAddFolder()
                }
                .buttonStyle(.borderedProminent)
                // White tint fill: dark label stays readable.
                .foregroundStyle(.black)
                .padding(.top, Layout.MusicGrid.emptyStateButtonTop)
            }
            Spacer()
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
