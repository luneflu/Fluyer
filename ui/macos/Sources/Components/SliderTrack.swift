import SwiftUI

/// Draggable horizontal slider used for both playback position and volume.
///
/// ponytail: the player bar and the now-playing card each had their own copy of this
/// widget (`VolumeBar` and `ProgressTrack`), differing only in track thickness.
struct SliderTrack: View {
    @Binding var fraction: Float

    /// The slim variant used for volume; the default is the seek bar.
    var isThin = false

    @State private var isHovering = false

    private var thickness: CGFloat {
        if isThin { return isHovering ? 5 : 4 }
        return isHovering ? 6 : 5
    }

    var body: some View {
        GeometryReader { geo in
            let width = geo.size.width
            let progress = CGFloat(fraction.clamped(to: 0...1))

            ZStack(alignment: .leading) {
                Capsule()
                    .fill(Color.white.opacity(0.18))
                    .frame(height: thickness)

                Capsule()
                    .fill(Color.white.opacity(isHovering ? 1.0 : 0.9))
                    .frame(width: max(0, width * progress), height: thickness)
            }
            .frame(maxHeight: .infinity, alignment: .center)
            .contentShape(Rectangle())
            .onHover { isHovering = $0 }
            .gesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { value in
                        // Guard the degenerate zero-width frame: `0 / 0` is NaN, and
                        // NaN would otherwise reach the seek conversion.
                        guard width > 0 else { return }
                        fraction = Float(value.location.x / width)
                    }
            )
        }
    }
}
