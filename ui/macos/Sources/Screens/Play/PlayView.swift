import SwiftUI
import FluyerCore

/// Full-screen now-playing surface, mirroring `Fluyer/src/routes/play/+page.svelte`.
///
/// ponytail: the Svelte page is a two-column grid — cover art (col 1, row 1), the
/// control card (col 1, row 2) and the lyrics scroller spanning both rows (col 2).
/// Without lyrics the page collapses to a single centered column. The proportions
/// below mirror `md:grid-cols-[40%_55%]` / `md:grid-cols-[50%]`.
struct PlayView: View {
    @Bindable var playback: PlaybackState
    let covers: CoverState

    @State private var isIdle = false
    @State private var backButtonHidden = false
    @State private var idleTask: Task<Void, Never>?

    /// Svelte hides the control-card column entirely below a single lyric line.
    private var hasLyrics: Bool { playback.playView.lyrics.count > 1 }

    var body: some View {
        GeometryReader { geo in
            ZStack(alignment: .topLeading) {
                HStack(alignment: .top, spacing: 0) {
                    // Single column instance for both states: the offset below is what
                    // slides it between centered (no lyrics) and left-rail (lyrics).
                    leftColumn(
                        columnWidth: geo.size.width * (hasLyrics ? Layout.Play.coverColumnRatio : Layout.Play.centeredColumnRatio),
                        isCentered: !hasLyrics
                    )
                    .padding(.top, Layout.Play.topPadding)

                    if hasLyrics {
                        LyricsView(playback: playback)
                            .frame(width: geo.size.width * Layout.Play.lyricsColumnRatio)
                            .transition(.move(edge: .trailing))
                    }
                }
                .padding(.trailing, Layout.Play.trailingPadding)
                .animation(.easeInOut(duration: 0.4), value: hasLyrics)

                backButton
            }
            .frame(width: geo.size.width, height: geo.size.height)
            .onContinuousHover { phase in
                if case .active = phase {
                    resetIdleTimer()
                }
            }
            .onExitCommand {
                handleBack()
            }
            .onAppear {
                resetIdleTimer()
            }
            .onDisappear {
                idleTask?.cancel()
            }
        }
    }

    // MARK: - Left Column (cover art + control card)

    /// Cover art and the control card share one width so the card always lines up with
    /// the artwork edge; the card hugs its own content vertically. Both are centred
    /// vertically: trailing-aligned against the lyrics column (`ms-auto` in the Svelte
    /// markup, with a 40pt gutter) when lyrics exist, centred on the window when not.
    ///
    /// ponytail: the horizontal placement is an explicit `offset` rather than a frame
    /// alignment, because alignment is resolved at layout time and would snap instead of
    /// sliding when lyrics appear or disappear on a track change.
    private func leftColumn(columnWidth: CGFloat, isCentered: Bool) -> some View {
        let side = min(max(columnWidth - Layout.Play.columnGutter, Layout.Play.minCoverSide), Layout.Play.maxCoverSide)
        let dx = isCentered ? (columnWidth - side) / 2 : max(columnWidth - Layout.Play.columnGutter - side, 0)

        return VStack(spacing: Layout.Play.coverToControls) {
            CurrentCoverView(playback: playback, covers: covers, side: side)
                .shadow(color: .black.opacity(0.35), radius: 16, x: 0, y: 8)

            PlayControlsView(playback: playback)
                .frame(width: side)
        }
        .frame(minWidth: columnWidth, maxWidth: columnWidth, maxHeight: .infinity, alignment: .leading)
        .offset(x: dx)
    }

    // MARK: - Back Button

    private var backButton: some View {
        Button(action: handleBack) {
            Image(systemName: "chevron.left")
                .font(.system(size: 13, weight: .semibold))
                .foregroundColor(.white)
                .frame(width: Layout.Play.backButtonSide, height: Layout.Play.backButtonSide)
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .help("Back (Esc)")
        .padding(.leading, Layout.Play.backButtonLeading)
        .padding(.top, Layout.Play.backButtonTop)
        .opacity(backButtonHidden ? 0 : (isIdle ? 0 : 0.7))
        .animation(.easeInOut(duration: 0.25), value: isIdle)
        .allowsHitTesting(!isIdle && !backButtonHidden)
    }

    private func handleBack() {
        guard !backButtonHidden else { return }
        backButtonHidden = true
        idleTask?.cancel()
        Task { @MainActor in
            try? await Task.sleep(nanoseconds: 300_000_000)
            withAnimation(.spring(response: 0.35, dampingFraction: 0.8)) {
                playback.showPlayView = false
            }
        }
    }

    private func resetIdleTimer() {
        if isIdle { isIdle = false }
        idleTask?.cancel()
        idleTask = Task { @MainActor in
            try? await Task.sleep(nanoseconds: 3_000_000_000)
            guard !Task.isCancelled else { return }
            isIdle = true
        }
    }
}
