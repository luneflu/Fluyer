import SwiftUI
import MetalKit
import FluyerCore

/// Animated, blurred artwork backdrop behind the whole window. With the
/// "Animated background" setting off, the Metal view is torn down and a static
/// blurred cover is shown instead.
struct AnimatedBackgroundView: View {
    @Bindable var state: AppState
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.scenePhase) private var scenePhase
    @State private var artwork: NSImage?

    private var artworkKey: String {
        let palette = state.playback.playView.palette
            .map { "\($0.r),\($0.g),\($0.b)" }
            .joined(separator: ";")
        return "\(state.playback.playView.track?.path ?? "none")|\(palette)"
    }

    var body: some View {
        Group {
            if state.settings.animatedBackground {
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
        .background(state.settings.animatedBackground ? Color(red: 0.2, green: 0.2, blue: 0.21) : .black)
        .allowsHitTesting(false)
        .accessibilityHidden(true)
        .task(id: artworkKey) {
            let key = artworkKey
            guard state.playback.playView.track != nil, let engine = state.engine else {
                artwork = nil
                return
            }
            let data = await engine.loadCurrentThumbnail(maxSize: Self.backdropPixels)
            guard !Task.isCancelled, key == artworkKey else { return }
            artwork = data.flatMap { NSImage(data: $0) }
        }
    }

    /// ponytail: the backdrop is full-screen and blurred to mush, so 1200px is ample
    /// and keeps the decode off the critical path.
    private static let backdropPixels: UInt32 = 1200
}

private struct MetalBackdropRepresentable: NSViewRepresentable {
    let image: NSImage?
    let reduceMotion: Bool
    let active: Bool

    func makeNSView(context: Context) -> MetalBackdropView { MetalBackdropView() }

    func updateNSView(_ view: MetalBackdropView, context: Context) {
        view.renderer?.setImage(image)
        view.renderer?.reduceMotion = reduceMotion
        view.isPaused = !active || view.renderer == nil
    }

    static func dismantleNSView(_ view: MetalBackdropView, coordinator: ()) {
        view.isPaused = true
        view.delegate = nil
    }
}

private final class MetalBackdropView: MTKView {
    var renderer: ArtworkBackdropRenderer?

    init() {
        super.init(frame: .zero, device: MTLCreateSystemDefaultDevice())
        colorPixelFormat = .bgra8Unorm
        preferredFramesPerSecond = 30
        clearColor = MTLClearColorMake(0.2, 0.2, 0.21, 1) // placeholder gray, pre-first-frame only
        do {
            guard let device else { throw BackdropError.unavailable }
            renderer = try ArtworkBackdropRenderer(device: device)
            delegate = renderer
        } catch {
            NSLog("Backdrop unavailable: %@", String(describing: error))
            isPaused = true
        }
    }

    required init(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }
}
