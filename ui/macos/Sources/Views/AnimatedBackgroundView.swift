import SwiftUI
import FluyerCore

/// Ambient blurred background for the now-playing screen.
///
/// Mirrors `AnimatedBackground.svelte`: the core builds a grid of randomly
/// palette-coloured blocks, gaussian-blurs it, and the UI stretches that over the
/// window. Frames cross-fade over 750ms, matching `TRANSITION_DURATION`.
///
/// ponytail: regenerate on track change and on a >=25% resize, the same triggers
/// the Svelte version uses, so dragging a window edge does not re-blur per frame.
struct AnimatedBackgroundView: View {
    @Bindable var state: AppState
    let size: CGSize

    @State private var currentFrame: NSImage?
    @State private var incomingFrame: NSImage?
    @State private var incomingOpacity: Double = 0
    @State private var lastRenderedSize: CGSize = .zero
    @State private var requestID = 0

    private static let transitionDuration: Double = 0.75

    /// Svelte watches both current music and palette-triggered settings changes.
    /// Include path so two tracks sharing a palette still cross-fade.
    private var backgroundKey: String {
        let palette = state.playView.palette.map { "\($0.r).\($0.g).\($0.b)" }.joined(separator: "-")
        return "\(state.playView.track?.path ?? "none")|\(palette)"
    }

    var body: some View {
        ZStack {
            Color.black
            if let currentFrame {
                image(currentFrame)
            }
            if let incomingFrame {
                image(incomingFrame).opacity(incomingOpacity)
            }
        }
        .task(id: requestID) {
            await loadFrame()
        }
        .onChange(of: backgroundKey) {
            requestID += 1
        }
        .onChange(of: size) {
            if resizedEnough() {
                requestID += 1
            }
        }
        .onAppear {
            lastRenderedSize = size
        }
    }

    private func image(_ image: NSImage) -> some View {
        Image(nsImage: image)
            .resizable()
            .interpolation(.high)
            .scaledToFill()
    }

    /// Same threshold as `onWindowResize` in the Svelte version.
    private func resizedEnough() -> Bool {
        guard lastRenderedSize.width > 0, lastRenderedSize.height > 0 else {
            lastRenderedSize = size
            return false
        }
        let widthDiff = abs(size.width - lastRenderedSize.width) / lastRenderedSize.width
        let heightDiff = abs(size.height - lastRenderedSize.height) / lastRenderedSize.height
        guard widthDiff >= 0.25 || heightDiff >= 0.25 else { return false }
        lastRenderedSize = size
        return true
    }

    @MainActor
    private func loadFrame() async {
        guard let engine = state.engine else { return }
        let width = UInt32(max(1, Int(size.width)))
        let height = UInt32(max(1, Int(size.height)))
        lastRenderedSize = size

        guard let frame = await engine.loadAnimatedBackground(width: width, height: height),
              let decoded = AnimatedBackgroundView.decode(frame)
        else { return }

        if currentFrame == nil {
            // First frame: fade up from black, as `triggerFadeIn` does.
            currentFrame = decoded
            withAnimation(.easeOut(duration: Self.transitionDuration)) {
                incomingFrame = decoded
                incomingOpacity = 1
            }
            await settle()
            return
        }

        // Track change: fade the new frame over the old one. An interrupted
        // transition just restarts from the current opacity, which is visually
        // equivalent to the Svelte canvas snapshot and needs no extra copy.
        incomingFrame = decoded
        incomingOpacity = 0
        withAnimation(.easeInOut(duration: Self.transitionDuration)) {
            incomingOpacity = 1
        }
        await settle()
    }

    private func settle() async {
        try? await Task.sleep(nanoseconds: UInt64(Self.transitionDuration * 1_000_000_000))
        guard incomingOpacity >= 1 else { return }
        currentFrame = incomingFrame
        incomingFrame = nil
        incomingOpacity = 0
    }

    /// Raw RGBA -> NSImage. `AnimatedBackgroundFrame` is tiny (5% of the window),
    /// so this stays cheap enough to stay off the main thread.
    private static func decode(_ frame: AnimatedBackgroundFrame) -> NSImage? {
        let width = Int(frame.width)
        let height = Int(frame.height)
        guard width > 0, height > 0 else { return nil }

        let bytesPerRow = width * 4
        guard let provider = CGDataProvider(data: frame.rgba as CFData),
              let cgImage = CGImage(
                  width: width,
                  height: height,
                  bitsPerComponent: 8,
                  bitsPerPixel: 32,
                  bytesPerRow: bytesPerRow,
                  space: CGColorSpaceCreateDeviceRGB(),
                  bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedLast.rawValue),
                  provider: provider,
                  decode: nil,
                  shouldInterpolate: true,
                  intent: .defaultIntent
              )
        else { return nil }

        return NSImage(cgImage: cgImage, size: .zero)
    }
}
