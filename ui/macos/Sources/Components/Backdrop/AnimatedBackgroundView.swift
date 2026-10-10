import SwiftUI
import MetalKit

/// Animated, blurred backdrop behind the whole window. Input is the cover or a
/// Rust-generated square of random palette blocks, per `settings.backdropSource`.
/// No track (or no cover) always uses Rust's grey blocks.
struct AnimatedBackgroundView: View {
    let playback: PlaybackState
    let settings: SettingsState
    let covers: CoverState
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.scenePhase) private var scenePhase
    @State private var artwork: NSImage?

    private var artworkKey: String {
        let palette = playback.playView.palette
            .map { "\($0.r),\($0.g),\($0.b)" }
            .joined(separator: ";")
        return "\(settings.backdropSource)|\(playback.playView.track?.path ?? "none")|\(palette)"
    }

    var body: some View {
        Group {
            if settings.animatedBackground {
                MetalBackdropRepresentable(
                    image: artwork,
                    reduceMotion: reduceMotion,
                    active: scenePhase == .active
                )
            } else if let artwork {
                Image(nsImage: artwork)
                    .resizable()
                    .scaledToFill()
                    .blur(radius: 60, opaque: true)
                    .overlay(Color.black.opacity(0.35))
                    .clipped()
            } else {
                Color.clear
            }
        }
        .background(settings.animatedBackground ? Color(red: 0.2, green: 0.2, blue: 0.21) : .black)
        .allowsHitTesting(false)
        .accessibilityHidden(true)
        .task(id: artworkKey) {
            let key = artworkKey
            let image = await covers.backdrop(source: settings.backdropSource, hasTrack: playback.playView.track != nil)
            guard !Task.isCancelled, key == artworkKey else { return }
            artwork = image
        }
    }
}
