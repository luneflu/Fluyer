import SwiftUI
import FluyerCore

/// Shell of the library window: backdrop, album carousel, track grid, player bar,
/// the queue sidebar, the now-playing overlay and the toast.
struct ContentView: View {
    @Bindable var state: AppState
    @State private var lastHover: CGPoint?
    @State private var edgeOpenTask: Task<Void, Never>?

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
            // Adding folders lives in File ▸ Open Music Folder (⌘O) and Settings.
            if state.library.scanStatus.isScanning {
                ToolbarItem(placement: .automatic) {
                    scanProgress
                }
            }
            // Search, then the queue toggle at the far right, in one item so the
            // order holds. `.primaryAction` is trailing in a titled, unified toolbar.
            if !state.playback.showPlayView {
                ToolbarItem(placement: .primaryAction) {
                    HStack(spacing: 8) {
                        ToolbarSearchField(
                            text: Bindable(state.selection).query,
                            prompt: "Search title, artist, album"
                        )
                        .frame(width: 260)
                        Toggle(isOn: Bindable(state.queue).isOpen) {
                            Label("Queue", systemImage: "list.bullet")
                        }
                        .toggleStyle(.button)
                        .labelStyle(.iconOnly)
                        .help("Queue (⌘L, or rest the pointer on the right edge)")
                        .accessibilityLabel(state.queue.isOpen ? "Hide Queue" : "Show Queue")
                    }
                    .fixedSize()
                }
            }
        }
        // Backdrop runs under the title bar; no toolbar material over it.
        .toolbarBackground(.hidden, for: .windowToolbar)
        // Empty title: the titled toolbar style keeps search + queue on the trailing
        // edge, but no "FluyerApp" text is drawn.
        .navigationTitle("")
        .onChange(of: state.playback.showPlayView) { _, showing in
            if showing { state.queue.isOpen = false }
        }
        // Esc closes the queue even when hover opened it and nothing has focus
        // (`onExitCommand`/`onKeyPress` need focus). `.hidden()` would disable the
        // shortcut, so the button is zero-size and transparent instead. While the
        // queue is open, the first Esc closes it; the next clears the search.
        .background {
            if state.queue.isOpen {
                Button("Close Queue") { state.queue.isOpen = false }
                    .keyboardShortcut(.cancelAction)
                    .frame(width: 0, height: 0)
                    .opacity(0)
                    .accessibilityHidden(true)
            }
        }
        .frame(minWidth: 920, minHeight: 680)
    }

    private var library: some View {
        VStack(spacing: 0) {
            // Sidebar.svelte: spans filter bar to player bar, 12pt outer padding,
            // two album slots wide.
            GeometryReader { geo in
                let sidebarWidth = AlbumCarouselView.sidebarWidth(forWidth: geo.size.width)
                ZStack(alignment: .trailing) {
                    VStack(spacing: 0) {
                        AlbumCarouselView(state: state)

                        if state.selection.isActive {
                            CollectionHeaderView(state: state)
                                .transition(.opacity.combined(with: .move(edge: .top)))
                        }

                        MusicGridView(state: state)
                            .frame(maxWidth: .infinity, maxHeight: .infinity)
                    }
                    // Cards behind the open queue fade out (useAlbumList / useMusicList).
                    .environment(\.sidebarMinX, state.queue.isOpen ? geo.size.width - 12 - sidebarWidth : .infinity)

                    if state.queue.isOpen {
                        QueuePaneView(state: state, width: sidebarWidth)
                            .padding([.trailing], 12)
                            .padding([.top], 6)
                            .padding([.bottom], 12)
                            .onExitCommand { state.queue.isOpen = false }
                            .transition(.move(edge: .trailing).combined(with: .opacity))
                            .zIndex(5)
                    }
                }
                .coordinateSpace(.named(SidebarOcclusion.coordinateSpace))
                // Sidebar.svelte edge trigger: rest on the right edge to open, move
                // away to close. The toolbar toggle and ⌘L stay for keyboard users.
                .onContinuousHover(coordinateSpace: .named(SidebarOcclusion.coordinateSpace)) { phase in
                    let trigger = EdgeTrigger(size: geo.size, panelMinX: geo.size.width - 12 - sidebarWidth - 12)
                    switch phase {
                    case .active(let point):
                        handleEdge(trigger.action(at: point, last: lastHover, isOpen: state.queue.isOpen))
                        lastHover = point
                    case .ended:
                        handleEdge(trigger.action(at: nil, last: lastHover, isOpen: state.queue.isOpen))
                        lastHover = nil
                    }
                }
            }

            PlayerBarView(state: state)
        }
        .animation(.easeInOut(duration: 0.2), value: state.selection.index)
        .animation(.easeInOut(duration: 0.5), value: state.queue.isOpen)
    }

    private func handleEdge(_ action: EdgeTrigger.Action) {
        switch action {
        case .none:
            break
        case .close:
            cancelEdgeOpen()
            state.queue.isOpen = false
        case .cancelOpen:
            cancelEdgeOpen()
        case .armOpen:
            // Skip while a mouse button is held: dragging the seek or volume bar
            // toward the right must not open the queue.
            guard edgeOpenTask == nil, NSEvent.pressedMouseButtons == 0 else { return }
            edgeOpenTask = Task {
                try? await Task.sleep(for: EdgeTrigger.openDelay)
                defer { edgeOpenTask = nil }
                guard !Task.isCancelled, !state.playback.showPlayView else { return }
                state.queue.isOpen = true
            }
        }
    }

    private func cancelEdgeOpen() {
        edgeOpenTask?.cancel()
        edgeOpenTask = nil
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

    private var scanProgress: some View {
        HStack(spacing: 6) {
            ProgressView()
                .controlSize(.small)
            Text(state.library.scanStatus.statusLabel)
                .font(.system(size: 11))
                .foregroundColor(.white.opacity(0.7))
        }
    }
}
