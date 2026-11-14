import SwiftUI
import FluyerCore

/// YouTube Music browser: the home feed. Anonymous, so no library or account state.
struct YoutubeMusicView: View {
    @Bindable var youtube: YoutubeMusicState
    @State private var isHovered = false

    var body: some View {
        content
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .onHover { isHovered = $0 }
            .task { await youtube.loadHome() }
    }

    /// Home feed: shelves of cover cards or track grids, scrolled vertically.
    @ViewBuilder
    private var content: some View {
        if youtube.shelves.isEmpty {
            message(youtube.isLoadingHome ? "Loading YouTube Music…"
                    : (youtube.error ?? "Nothing to show"))
        } else {
            ScrollView {
                VStack(alignment: .leading, spacing: Layout.YoutubeMusic.shelfSpacing) {
                    ForEach(Array(youtube.shelves.enumerated()), id: \.offset) { _, shelf in
                        YoutubeShelfView(shelf: shelf, playingVideoId: youtube.playingVideoId) { item in
                            Task { await youtube.play(item) }
                        }
                    }
                }
                .padding(.vertical, Layout.YoutubeMusic.shelfVerticalPadding)
            }
            .scrollIndicators(.hidden)
        }
    }

    private func message(_ text: String) -> some View {
        Text(text)
            .font(.system(size: 14))
            .foregroundColor(.white.opacity(0.5))
            .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
