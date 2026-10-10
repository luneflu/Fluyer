import SwiftUI
import FluyerCore

/// Title strip and album actions, shown between the carousel and the grid while an
/// album is selected.
struct AlbumHeaderView: View {
    @Bindable var filter: LibraryFilterState

    var body: some View {
        HStack(spacing: Layout.AlbumHeader.spacing) {
            Text(filter.detail?.subtitle ?? "")
                .font(.system(size: 13, weight: .medium))
                .foregroundColor(.white)
                .lineLimit(1)

            Spacer()

            HStack(spacing: Layout.AlbumHeader.buttonSpacing) {
                iconButton("chevron.left", help: "Back to Albums") {
                    filter.clear()
                }

                iconButton("play.fill", help: "Play Album") {
                    filter.playSelected()
                }

                iconButton("text.badge.plus", help: "Add to Queue") {
                    filter.queueSelected()
                }

                iconButton("shuffle", help: "Shuffle Album") {
                    filter.shuffleSelected()
                }
            }
        }
        .padding(.horizontal, Layout.AlbumHeader.horizontalPadding)
        .frame(height: Layout.AlbumHeader.height)
        .background(
            RoundedRectangle(cornerRadius: Layout.AlbumHeader.cornerRadius)
                .fill(Color.white.opacity(0.08))
                .overlay(
                    RoundedRectangle(cornerRadius: Layout.AlbumHeader.cornerRadius)
                        .stroke(Color.white.opacity(0.12), lineWidth: 1)
                )
        )
        .padding(.horizontal, Layout.AlbumHeader.outerHorizontalPadding)
        .padding(.top, Layout.AlbumHeader.outerTopPadding)
    }

    private func iconButton(
        _ icon: String,
        size: CGFloat = 13,
        help: String,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Image(systemName: icon)
                .font(.system(size: size, weight: .semibold))
                .frame(width: Layout.AlbumHeader.buttonSide, height: Layout.AlbumHeader.buttonSide)
                .foregroundColor(.white)
        }
        .buttonStyle(.plain)
        .help(help)
    }
}
