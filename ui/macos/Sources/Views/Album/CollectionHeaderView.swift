import SwiftUI
import FluyerCore

/// Title strip and album actions, shown between the carousel and the grid while an
/// album is selected.
struct CollectionHeaderView: View {
    @Bindable var state: AppState

    var body: some View {
        HStack(spacing: 12) {
            Text(state.selection.detail?.subtitle ?? "")
                .font(.system(size: 13, weight: .medium))
                .foregroundColor(.white)
                .lineLimit(1)

            Spacer()

            HStack(spacing: 8) {
                iconButton("chevron.left", help: "Back to Albums") {
                    state.selection.clear()
                }

                iconButton("play.fill", help: "Play Album") {
                    state.selection.playSelected()
                }

                iconButton("text.badge.plus", help: "Add to Queue") {
                    state.selection.queueSelected()
                }

                iconButton("shuffle", help: "Shuffle Album") {
                    state.selection.shuffleSelected()
                }
            }
        }
        .padding(.horizontal, 16)
        .frame(height: 46)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(Color.white.opacity(0.08))
                .overlay(
                    RoundedRectangle(cornerRadius: 8)
                        .stroke(Color.white.opacity(0.12), lineWidth: 1)
                )
        )
        .padding(.horizontal, 16)
        .padding(.top, 4)
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
                .frame(width: 28, height: 28)
                .foregroundColor(.white)
        }
        .buttonStyle(.plain)
        .help(help)
    }
}
