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
        .clipShape(RoundedRectangle(cornerRadius: side * 0.15))
        .shadow(color: .black.opacity(0.2), radius: 3, x: 0, y: 1)
        .task(id: trackPath) {
            guard let engine = state.engine else { return }
            let maxPx = UInt32(max(32, Int(side * 2)))
            let data = await engine.loadCurrentThumbnail(maxSize: maxPx)
            guard let data, !data.isEmpty else {
                thumbnail = nil
                return
            }
            thumbnail = await ThumbnailStore.shared.image(
                key: "current-\(maxPx)",
                maxPixelSize: Int(maxPx),
                load: { data }
            )
        }
    }
}
