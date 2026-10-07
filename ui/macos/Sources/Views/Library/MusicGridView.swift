import SwiftUI
import FluyerCore

/// Adaptive grid of track rows. Shows the whole library, or one album's tracks when
/// `AlbumSelection` is active.
struct MusicGridView: View {
    @Bindable var state: AppState

    private let columns = [
        GridItem(.adaptive(minimum: 280, maximum: .infinity), spacing: 12)
    ]

    var body: some View {
        let tracks = state.selection.displayedTracks

        Group {
            if tracks.isEmpty {
                emptyState
            } else {
                ScrollView {
                    LazyVGrid(columns: columns, spacing: 8) {
                        ForEach(tracks, id: \.path) { track in
                            TrackCard(state: state, track: track)
                                .hiddenBySidebar()
                        }
                    }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 8)
                }
            }
        }
    }

    private var emptyState: some View {
        VStack(spacing: 12) {
            Spacer()
            Image(systemName: "music.note.list")
                .font(.system(size: 36))
                .foregroundColor(.white.opacity(0.2))
            Text(state.library.tracks.isEmpty ? "No songs in library" : "No songs found")
                .font(.system(size: 14))
                .foregroundColor(.white.opacity(0.5))
            if state.library.tracks.isEmpty {
                Button("Open Music Folder...") {
                    state.promptAddFolder()
                }
                .buttonStyle(.borderedProminent)
                .padding(.top, 4)
            }
            Spacer()
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

// MARK: - Row

private struct TrackCard: View {
    @Bindable var state: AppState
    let track: TrackItemViewModel

    @State private var thumbnail: NSImage?

    private var isCurrent: Bool { track.isCurrent }

    /// Inside an album every row renders the album cover, so they share one cache
    /// entry; otherwise each row uses its own track cover.
    private var cacheKey: String {
        let pixels = Self.pixels
        if let albumIndex = state.selection.index {
            return ThumbnailKey.album(UInt64(albumIndex), px: pixels)
        }
        return ThumbnailKey.track(track.index, px: pixels)
    }

    private static let pixels = 88

    var body: some View {
        HStack(spacing: 10) {
            thumbnailView

            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    if isCurrent {
                        Image(systemName: state.playback.bar.isPlaying ? "speaker.wave.2.fill" : "speaker.fill")
                            .font(.system(size: 10))
                            .foregroundColor(.accentColor)
                    }

                    Text(track.title)
                        .font(.system(size: 13, weight: isCurrent ? .semibold : .regular))
                        .foregroundColor(isCurrent ? .accentColor : .white)
                        .lineLimit(1)
                }

                Text("\(track.artist) • \(track.album)")
                    .font(.system(size: 11))
                    .foregroundColor(.white.opacity(0.6))
                    .lineLimit(1)
            }

            Spacer(minLength: 4)

            Text(track.durationFormatted)
                .font(.system(size: 11, design: .monospaced))
                .foregroundColor(.white.opacity(0.4))
        }
        .padding(.vertical, 6)
        .contentShape(Rectangle())
        .onTapGesture {
            state.selection.playTrack(track)
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
        .frame(width: 44, height: 44)
    }

    private func loadThumbnail() async {
        guard let engine = state.engine else { return }

        // ponytail: no peek fast-path here — `ThumbnailStore.image` peeks first and
        // returns before awaiting the core, so the extra call was pure duplication.
        let pixels = UInt32(Self.pixels)
        let data: Data?
        if let albumIndex = state.selection.index {
            data = await engine.loadAlbumThumbnail(index: UInt64(albumIndex), maxSize: pixels)
        } else {
            data = await engine.loadTrackThumbnail(index: track.index, maxSize: pixels)
        }

        guard let data, !data.isEmpty else { return }
        thumbnail = await ThumbnailStore.shared.image(key: cacheKey) { data }
    }
}
