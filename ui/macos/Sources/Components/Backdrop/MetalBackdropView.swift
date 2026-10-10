import MetalKit

final class MetalBackdropView: MTKView {
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
