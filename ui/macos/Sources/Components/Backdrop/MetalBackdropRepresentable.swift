import SwiftUI
import MetalKit

struct MetalBackdropRepresentable: NSViewRepresentable {
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
