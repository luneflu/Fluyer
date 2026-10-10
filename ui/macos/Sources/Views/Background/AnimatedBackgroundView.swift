import SwiftUI
import MetalKit
import FluyerCore

/// Animated, blurred backdrop behind the whole window. Input is the cover or a
/// Rust-generated square of random palette blocks, per `settings.backdropSource`.
/// No track (or no cover) always uses Rust's grey blocks.
struct AnimatedBackgroundView: View {
    @Bindable var state: AppState
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.scenePhase) private var scenePhase
    @State private var artwork: NSImage?

    private var artworkKey: String {
        let palette = state.playback.playView.palette
            .map { "\($0.r),\($0.g),\($0.b)" }
            .joined(separator: ";")
        return "\(state.settings.backdropSource)|\(state.playback.playView.track?.path ?? "none")|\(palette)"
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
            guard let engine = state.engine else {
                artwork = nil
                return
            }
            var image: NSImage?
            if state.settings.backdropSource == .artwork, state.playback.playView.track != nil {
                image = await engine.artworkLoadCurrentThumbnail(maxSize: Self.backdropPixels).flatMap { NSImage(data: $0) }
            }
            // Blocks mode, no track, or coverless track: Rust gives palette/grey blocks.
            if image == nil {
                image = Self.image(from: await engine.backdropLoadBlockArtwork())
            }
            guard !Task.isCancelled, key == artworkKey else { return }
            artwork = image
        }
    }

    /// ponytail: the backdrop is full-screen and blurred to mush, so 1200px is ample
    /// and keeps the decode off the critical path.
    private static let backdropPixels: UInt32 = 1200

    private static func image(from frame: AnimatedBackgroundFrame) -> NSImage? {
        let w = Int(frame.width), h = Int(frame.height)
        guard w > 0, h > 0, frame.rgba.count == w * h * 4,
              let provider = CGDataProvider(data: Data(frame.rgba) as CFData),
              let cg = CGImage(width: w, height: h, bitsPerComponent: 8, bitsPerPixel: 32, bytesPerRow: w * 4,
                               space: CGColorSpace(name: CGColorSpace.sRGB)!,
                               bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.noneSkipLast.rawValue),
                               provider: provider, decode: nil, shouldInterpolate: true, intent: .defaultIntent)
        else { return nil }
        return NSImage(cgImage: cg, size: NSSize(width: w, height: h))
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
