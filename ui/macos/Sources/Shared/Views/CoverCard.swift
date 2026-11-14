import SwiftUI

/// Square cover with title/artist labels, shared by the local album shelves and the
/// YouTube Music feed. Only the image source differs, so it is injected:
/// `cached` answers directly from a warm cache, `load` fetches at the given pixel size.
struct CoverCard: View {
    let coverSize: CGFloat
    let title: String
    let subtitle: String?
    var isSelected = false
    var isDimmed = false
    var actions: [(title: String, run: () -> Void)] = []
    var onTap: (() -> Void)?
    let cached: (Int) -> NSImage?
    let load: (Int) async -> NSImage?

    @State private var thumbnail: NSImage?
    @State private var isHovered = false

    /// Requested at 2x for Retina, which is also what makes this key distinct from the
    /// 88px thumbnail the track rows ask for.
    private var pixelSize: Int { max(16, Int(coverSize * 2)) }

    var body: some View {
        VStack(alignment: .leading, spacing: Layout.Albums.coverToLabels) {
            cover
            labels
        }
        .frame(width: coverSize)
        .opacity(isDimmed ? 0.4 : 1)
        .animation(.easeInOut(duration: 0.3), value: isDimmed)
        .contentShape(Rectangle())
        .onHover { isHovered = $0 }
        .onTapGesture { onTap?() }
        .contextMenu {
            ForEach(actions, id: \.title) { action in
                Button(action.title, action: action.run)
            }
        }
        .accessibilityElement(children: .combine)
        .accessibilityAddTraits(.isButton)
        .accessibilityAddTraits(isSelected ? .isSelected : [])
        .task(id: pixelSize) {
            // Warm cache: show at once. Cold: fade in once decoded (`anim-fade-in`).
            if let hit = cached(pixelSize) {
                thumbnail = hit
                return
            }
            guard let image = await load(pixelSize) else { return }
            withAnimation(.easeIn(duration: 0.3)) { thumbnail = image }
        }
    }

    private var cover: some View {
        ZStack {
            // No-cover / not-yet-loaded fallback. Hidden once the real cover is in,
            // so it never sits behind the bitmap.
            if thumbnail == nil {
                Rectangle()
                    .fill(Color.white.opacity(0.08))
                    .overlay(
                        Image(systemName: "music.note")
                            .font(.system(size: coverSize * 0.25))
                            .foregroundColor(.white.opacity(0.3))
                    )
            }
            if let thumbnail {
                Image(nsImage: thumbnail)
                    .resizable()
                    .aspectRatio(1, contentMode: .fill)
                    .transition(.opacity)
            }
            // Selected: solid border. Others: border + white wash on hover.
            RoundedRectangle(cornerRadius: Layout.Albums.coverCornerRadius)
                .fill(Color.white.opacity(isSelected ? 0 : 0.2))
                .strokeBorder(Color.white, lineWidth: 2)
                .opacity(isSelected || isHovered ? 1 : 0)
                .animation(.easeInOut(duration: isHovered ? 0.5 : 0.75), value: isHovered)
        }
        .frame(width: coverSize, height: coverSize)
        .clipShape(RoundedRectangle(cornerRadius: Layout.Albums.coverCornerRadius))
        .shadow(color: .black.opacity(0.25), radius: 6, x: 0, y: 3)
    }

    private var labels: some View {
        VStack(alignment: .leading, spacing: Layout.Albums.nameToArtist) {
            Text(title)
                .font(.system(size: 13, weight: isSelected ? .semibold : .medium))
                .foregroundColor(.white)
                .lineLimit(1)

            if let subtitle {
                Text(subtitle)
                    .font(.system(size: 11))
                    .foregroundColor(.white.opacity(0.6))
                    .lineLimit(1)
            }
        }
        .frame(width: coverSize, alignment: .leading)
    }
}
