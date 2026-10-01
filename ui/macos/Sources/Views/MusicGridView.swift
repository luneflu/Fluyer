import SwiftUI
import FluyerCore

public struct MusicGridView: View {
    @Bindable var state: AppState

    private let columns = [
        GridItem(.adaptive(minimum: 280, maximum: .infinity), spacing: 12)
    ]

    public var body: some View {
        let tracks = state.displayedTracks

        Group {
            if tracks.isEmpty {
                VStack(spacing: 12) {
                    Spacer()
                    Image(systemName: "music.note.list")
                        .font(.system(size: 36))
                        .foregroundColor(.white.opacity(0.2))
                    Text(state.tracks.isEmpty ? "No songs in library" : "No songs found")
                        .font(.system(size: 14))
                        .foregroundColor(.white.opacity(0.5))
                    if state.tracks.isEmpty {
                        Button("Open Music Folder...") {
                            state.promptAddFolder()
                        }
                        .buttonStyle(.borderedProminent)
                        .padding(.top, 4)
                    }
                    Spacer()
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    LazyVGrid(columns: columns, spacing: 8) {
                        ForEach(Array(tracks.enumerated()), id: \.element.index) { pair in
                            TrackCard(state: state, track: pair.element, rowIndex: pair.offset)
                        }
                    }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 8)
                }
            }
        }
    }
}

// MARK: - Row

private struct TrackCard: View {
    @Bindable var state: AppState
    let track: TrackItemViewModel
    let rowIndex: Int

    @State private var thumbnail: NSImage?
    @State private var didLoad = false

    private var isCurrent: Bool { track.isCurrent }

    private var cacheKey: String {
        if let albumIdx = state.selectedAlbumIndex {
            return "album-\(albumIdx)"
        }
        return "track-\(track.index)"
    }

    var body: some View {
        HStack(spacing: 10) {
            thumbnailView

            // Metadata text
            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    if isCurrent {
                        Image(systemName: state.playerBar.isPlaying ? "speaker.wave.2.fill" : "speaker.fill")
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

            // Duration
            Text(track.durationFormatted)
                .font(.system(size: 11, design: .monospaced))
                .foregroundColor(.white.opacity(0.4))
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(isCurrent ? Color.white.opacity(0.12) : Color.white.opacity(0.04))
        )
        .contentShape(Rectangle())
        .onTapGesture {
            state.playTrack(at: rowIndex)
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
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }

    private func loadThumbnail() async {
        guard let engine = state.engine else { return }

        // Fast path: already decoded (row recycled by LazyVGrid).
        if let hit = ThumbnailStore.shared.peek(cacheKey) {
            thumbnail = hit
            return
        }

        let data: Data?
        if let albumIdx = state.selectedAlbumIndex {
            data = await engine.loadAlbumThumbnail(index: UInt64(albumIdx), maxSize: 88)
        } else {
            data = await engine.loadTrackThumbnail(index: track.index, maxSize: 88)
        }

        guard let data, !data.isEmpty else {
            didLoad = true
            return
        }
        thumbnail = await ThumbnailStore.shared.image(
            key: cacheKey,
            maxPixelSize: 88,
            load: { data }
        )
        didLoad = true
    }
}
