import SwiftUI
import FluyerCore

/// Shell of the library window: backdrop, album carousel, track grid, player bar,
/// the queue sidebar, the now-playing overlay and the toast.
struct HomeView: View {
    @Bindable var state: AppState
    @State private var lastHover: CGPoint?
    @State private var edgeOpenTask: Task<Void, Never>?

    var body: some View {
        ZStack {
            AnimatedBackgroundView(playback: state.playback, settings: state.settings, covers: state.covers)
                .ignoresSafeArea()

            if !state.playback.showPlayView {
                library
            }

            if state.playback.showPlayView {
                PlayView(playback: state.playback, covers: state.covers)
                    .transition(.opacity)
                    .zIndex(10)
            }

            if let message = state.toast.message {
                ToastView(message: message)
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
                    HStack(spacing: Layout.Window.toolbarSpacing) {
                        Picker("View", selection: Bindable(state.selection).mode) {
                            Label("Songs", systemImage: "music.note.list").tag(LibraryMode.tracks)
                            Label("Albums", systemImage: "square.grid.2x2").tag(LibraryMode.albums)
                        }
                        .pickerStyle(.segmented)
                        .labelStyle(.iconOnly)
                        .help("Songs (⌘1) / Albums (⌘2)")
                        SortMenu(selection: state.selection)
                        ToolbarSearchField(
                            text: Bindable(state.selection).query,
                            prompt: "Search title, artist, album"
                        )
                        .frame(width: Layout.Window.searchFieldWidth)
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
        .frame(minWidth: Layout.Window.minWidth, minHeight: Layout.Window.minHeight)
    }

    private var library: some View {
        VStack(spacing: 0) {
            // Sidebar.svelte: spans filter bar to player bar, 12pt outer padding,
            // two album slots wide.
            GeometryReader { geo in
                let sidebarWidth = AlbumMetrics.sidebarWidth(forWidth: geo.size.width)
                ZStack(alignment: .trailing) {
                    VStack(spacing: 0) {
                        switch state.selection.mode {
                        case .albums:
                            AlbumGridView(filter: state.selection, library: state.library, covers: state.covers)
                                .transition(.opacity)
                        case .tracks:
                            AlbumCarouselView(filter: state.selection, covers: state.covers)

                            if state.selection.isActive {
                                AlbumHeaderView(filter: state.selection)
                                    .transition(.opacity.combined(with: .move(edge: .top)))
                            }

                            MusicGridView(filter: state.selection, library: state.library, playback: state.playback,
                                          covers: state.covers, onAddFolder: state.promptAddFolder)
                                .frame(maxWidth: .infinity, maxHeight: .infinity)
                        }
                    }
                    // Cards behind the open queue fade out (useAlbumList / useMusicList).
                    .environment(\.sidebarMinX, state.queue.isOpen ? geo.size.width - Layout.Queue.outerInset - sidebarWidth : .infinity)

                    if state.queue.isOpen {
                        QueueView(queue: state.queue, playback: state.playback, library: state.library,
                                  covers: state.covers, width: sidebarWidth)
                            .padding([.trailing], Layout.Queue.outerInset)
                            .padding([.top], Layout.Queue.outerTop)
                            .padding([.bottom], Layout.Queue.outerInset)
                            .onExitCommand { state.queue.isOpen = false }
                            .transition(.move(edge: .trailing).combined(with: .opacity))
                            .zIndex(5)
                    }
                }
                .coordinateSpace(.named(SidebarOcclusion.coordinateSpace))
                // Sidebar.svelte edge trigger: rest on the right edge to open, move
                // away to close. The toolbar toggle and ⌘L stay for keyboard users.
                .onContinuousHover(coordinateSpace: .named(SidebarOcclusion.coordinateSpace)) { phase in
                    let trigger = QueueEdgeTrigger(size: geo.size, panelMinX: geo.size.width - Layout.Queue.outerInset * 2 - sidebarWidth)
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

            PlayerBarView(playback: state.playback, covers: state.covers)
        }
        .animation(.easeInOut(duration: 0.2), value: state.selection.index)
        .animation(.easeInOut(duration: 0.3), value: state.selection.mode)
        .animation(.easeInOut(duration: 0.5), value: state.queue.isOpen)
    }

    private func handleEdge(_ action: QueueEdgeTrigger.Action) {
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
                try? await Task.sleep(for: QueueEdgeTrigger.openDelay)
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

    private var scanProgress: some View {
        HStack(spacing: Layout.Window.scanProgressSpacing) {
            ProgressView()
                .controlSize(.small)
            Text(state.library.scanStatus.statusLabel)
                .font(.system(size: 11))
                .foregroundColor(.white.opacity(0.7))
        }
    }
}
