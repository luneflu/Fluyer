import SwiftUI
import FluyerCore

public struct ContentView: View {
    @Bindable var state: AppState

    public var body: some View {
        ZStack {
            GeometryReader { geometry in
                AnimatedBackgroundView(state: state, size: geometry.size)
            }
                .ignoresSafeArea()

            VStack(spacing: 0) {
                // Top section: Album Carousel
                AlbumCarouselView(state: state)

                // Collection Header (shown when an album is selected)
                if state.selectedAlbumIndex != nil {
                    CollectionHeaderView(state: state)
                        .transition(.opacity.combined(with: .move(edge: .top)))
                }

                // Middle section: Responsive Music Grid matching wxWidgets MusicListCtrl
                MusicGridView(state: state)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)

                // Bottom section: Player Bar matching wxWidgets PlayerBarCtrl
                PlayerBarView(state: state)
            }
            .animation(.easeInOut(duration: 0.2), value: state.selectedAlbumIndex)

            // Fullscreen PlayView overlay matching wxWidgets ShowPlayView(true)
            if state.showPlayView {
                PlayView(state: state)
                    .transition(.opacity)
                    .zIndex(10)
            }

            // Toast notification
            if let msg = state.toastMessage {
                VStack {
                    Spacer()
                    Text(msg)
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
        }
        .toolbar {
            ToolbarItem(placement: .automatic) {
                if state.scanStatus.isScanning {
                    HStack(spacing: 6) {
                        ProgressView()
                            .controlSize(.small)
                        Text(state.scanStatus.statusLabel)
                            .font(.system(size: 11))
                            .foregroundColor(.white.opacity(0.7))
                    }
                } else {
                    Button(action: { state.promptAddFolder() }) {
                        Label("Open Music Folder...", systemImage: "folder")
                    }
                    .help("Open Music Folder (Ctrl+O)")
                }
            }
        }
        .frame(minWidth: 920, minHeight: 680)
    }

}
