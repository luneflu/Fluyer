import SwiftUI
import MetalKit
import FluyerCore

struct AnimatedBackgroundView: View {
    @Bindable var state: AppState
    let size: CGSize
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.scenePhase) private var scenePhase
    @State private var artwork: NSImage?

    private var artworkKey: String {
        let palette = state.playView.palette.map { "\($0.r),\($0.g),\($0.b)" }.joined(separator: ";")
        return "\(state.playView.track?.path ?? "none")|\(palette)"
    }

    var body: some View {
        MetalBackdropRepresentable(image: artwork, reduceMotion: reduceMotion,
                                   active: scenePhase == .active)
            .background(Color.black)
            .allowsHitTesting(false)
            .accessibilityHidden(true)
            .task(id: artworkKey) {
                let key = artworkKey
                guard state.playView.track != nil, let engine = state.engine else {
                    artwork = nil
                    return
                }
                let data = await engine.loadCurrentThumbnail(maxSize: 1200)
                guard !Task.isCancelled, key == artworkKey else { return }
                artwork = data.flatMap { NSImage(data: $0) }
            }
    }
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
        clearColor = MTLClearColorMake(0.07, 0.07, 0.09, 1)
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
