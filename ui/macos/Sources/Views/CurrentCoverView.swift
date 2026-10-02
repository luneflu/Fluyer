import SwiftUI
import FluyerCore

/// Shared cover-art view used by the player bar and the now-playing screen.
///
/// ponytail: loading is keyed on the playing track path so the bitmap is fetched
/// once per song change and reused by both surfaces.
struct CurrentCoverView: View {
    @Bindable var state: AppState
    let side: CGFloat

    @State private var thumbnail: NSImage?

    // ponytail: keyed on the published track path, not a fresh FFI round-trip.
    private var trackPath: String {
        state.playView.track?.path ?? "none"
    }

    // ponytail: the cache key must carry the track identity. Keying on the pixel size
    // alone (`current-720`) made every track change hit the LRU and keep rendering the
    // previous song's artwork, and the size-based invalidation in AppState only ever
    // cleared two hardcoded sizes that no caller used.
    private var cacheKey: String {
        "current-\(Int(side))-\(trackPath)"
    }

    var body: some View {
        ZStack {
            if let thumbnail {
                Image(nsImage: thumbnail)
                    .resizable()
                    .aspectRatio(contentMode: .fill)
            } else {
                Rectangle()
                    .fill(Color.white.opacity(0.1))
                    .overlay(
                        Image(systemName: "music.note")
                            .font(.system(size: side * 0.4))
                            .foregroundColor(.white.opacity(0.4))
                    )
            }
        }
        .frame(width: side, height: side)
        // .clipShape(RoundedRectangle(cornerRadius: side * 0.15))
        .shadow(color: .black.opacity(0.2), radius: 3, x: 0, y: 1)
        .task(id: trackPath) {
            guard let engine = state.engine else { return }
            let maxPx = UInt32(max(32, Int(side * 2)))
            let loaded = await ThumbnailStore.shared.image(
                key: cacheKey,
                maxPixelSize: Int(maxPx),
                load: { await engine.loadCurrentThumbnail(maxSize: maxPx) }
            )
            // A cancelled task can still resume after its `await`; drop the result so a
            // slow load from the previous track cannot overwrite the new artwork.
            guard !Task.isCancelled else { return }
            thumbnail = loaded
        }
    }
}
