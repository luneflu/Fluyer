import SwiftUI
import FluyerCore

public struct CollectionHeaderView: View {
    @Bindable var state: AppState

    public var body: some View {
        HStack(spacing: 12) {
            // Left: Album Label
            Text(state.selectedAlbum?.subtitle ?? "")
                .font(.system(size: 13, weight: .medium))
                .foregroundColor(.white)
                .lineLimit(1)

            Spacer()

            // Right: 4 Action Buttons
            HStack(spacing: 8) {
                Button(action: { state.clearAlbumSelection() }) {
                    Image(systemName: "chevron.left")
                        .font(.system(size: 13, weight: .semibold))
                        .frame(width: 28, height: 28)
                        .foregroundColor(.white)
                }
                .buttonStyle(.plain)
                .help("Back to Albums")

                Button(action: { state.playSelectedAlbum() }) {
                    Image(systemName: "play.fill")
                        .font(.system(size: 13, weight: .semibold))
                        .frame(width: 28, height: 28)
                        .foregroundColor(.white)
                }
                .buttonStyle(.plain)
                .help("Play Album")

                Button(action: { state.queueSelectedAlbum() }) {
                    Image(systemName: "text.badge.plus")
                        .font(.system(size: 14))
                        .frame(width: 28, height: 28)
                        .foregroundColor(.white)
                }
                .buttonStyle(.plain)
                .help("Add to Queue")

                Button(action: { state.shuffleSelectedAlbum() }) {
                    Image(systemName: "shuffle")
                        .font(.system(size: 13))
                        .frame(width: 28, height: 28)
                        .foregroundColor(.white)
                }
                .buttonStyle(.plain)
                .help("Shuffle Album")
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
}
