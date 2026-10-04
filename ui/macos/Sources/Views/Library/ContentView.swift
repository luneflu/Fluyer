import SwiftUI
import FluyerCore

/// Shell of the library window: backdrop, album carousel, track grid, player bar,
/// the now-playing overlay and the toast.
struct ContentView: View {
    @Bindable var state: AppState

    var body: some View {
        ZStack {
            AnimatedBackgroundView(state: state)
                .ignoresSafeArea()

            if !state.playback.showPlayView {
                library
            }

            if state.playback.showPlayView {
                PlayView(state: state)
                    .transition(.opacity)
                    .zIndex(10)
            }

            if let message = state.toast.message {
                toast(message)
            }
        }
        .toolbar {
            ToolbarItem(placement: .automatic) {
                toolbarItem
            }
        }
        .frame(minWidth: 920, minHeight: 680)
    }

    private var library: some View {
        VStack(spacing: 0) {
            AlbumCarouselView(state: state)

            if state.selection.isActive {
                CollectionHeaderView(state: state)
                    .transition(.opacity.combined(with: .move(edge: .top)))
            }

            MusicGridView(state: state)
                .frame(maxWidth: .infinity, maxHeight: .infinity)

            PlayerBarView(state: state)
        }
        .animation(.easeInOut(duration: 0.2), value: state.selection.index)
    }

    private func toast(_ message: String) -> some View {
        VStack {
            Spacer()
            Text(message)
                .font(.system(size: 13, weight: .medium))
                .foregroundColor(.white)
                .padding(.horizontal, 16)
                .padding(.vertical, 10)
                .background(.ultraThinMaterial)
                .clipShape(Capsule())
                .shadow(color: .black.opacity(0.3), radius: 8, x: 0, y: 4)
                .padding(.bottom, 80)
                .transition(.move(edge: .bottom).combined(with: .opacity))
        }
        .zIndex(20)
    }

    @ViewBuilder
    private var toolbarItem: some View {
        if state.library.scanStatus.isScanning {
            HStack(spacing: 6) {
                ProgressView()
                    .controlSize(.small)
                Text(state.library.scanStatus.statusLabel)
                    .font(.system(size: 11))
                    .foregroundColor(.white.opacity(0.7))
            }
        } else {
            Button(action: { state.promptAddFolder() }) {
                Label("Open Music Folder...", systemImage: "folder")
            }
            .help("Open Music Folder")
        }
    }
}
