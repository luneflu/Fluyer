import SwiftUI
import FluyerCore

/// Full-screen now-playing surface, mirroring `Fluyer/src/routes/play/+page.svelte`.
///
/// ponytail: the Svelte page is a two-column grid — cover art (col 1, row 1), the
/// control card (col 1, row 2) and the lyrics scroller spanning both rows (col 2).
/// Without lyrics the page collapses to a single centered column. The proportions
/// below mirror `md:grid-cols-[40%_55%]` / `md:grid-cols-[50%]`.
struct PlayView: View {
    @Bindable var state: AppState

    @State private var isIdle = false
    @State private var backButtonHidden = false
    @State private var idleTask: Task<Void, Never>?

    /// Svelte hides the control-card column entirely below a single lyric line.
    private var hasLyrics: Bool { state.playback.playView.lyrics.count > 1 }

    private var activeLyricIndex: Int { state.playback.clock.currentLyricIndex }

    var body: some View {
        GeometryReader { geo in
            ZStack(alignment: .topLeading) {
                HStack(alignment: .top, spacing: 0) {
                    // Single column instance for both states: the offset below is what
                    // slides it between centered (no lyrics) and left-rail (lyrics).
                    leftColumn(
                        columnWidth: geo.size.width * (hasLyrics ? 0.40 : 0.50),
                        isCentered: !hasLyrics
                    )
                    .padding(.top, 24)

                    if hasLyrics {
                        lyricsColumn
                            .frame(width: geo.size.width * 0.55)
                            .transition(.move(edge: .trailing))
                    }
                }
                .padding(.trailing, 20)
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
        let side = min(max(columnWidth - 40, 200), 360)
        let dx = isCentered ? (columnWidth - side) / 2 : max(columnWidth - 40 - side, 0)

        return VStack(spacing: 16) {
            CurrentCoverView(state: state, side: side)
                .shadow(color: .black.opacity(0.35), radius: 16, x: 0, y: 8)

            controlCard
                .frame(width: side)
        }
        .frame(minWidth: columnWidth, maxWidth: columnWidth, maxHeight: .infinity, alignment: .leading)
        .offset(x: dx)
    }

    private var controlCard: some View {
        VStack(spacing: 0) {
            trackInfoRow
                .padding(.top, 4)
                .padding(.bottom, 12)

            SliderTrack(
                fraction: Binding(
                    get: { state.playback.playerBar.progressPct },
                    set: { state.playback.seek(toFraction: $0) }
                )
            )
            .frame(height: 8)

            transportControls
                .padding(.top, 4)

            volumeRow
        }
        .fixedSize(horizontal: false, vertical: true)
    }

    /// Elapsed time on the left, `artist • title` centred, total duration right.
    ///
    /// ponytail: `.transition(.identity)` keeps the metadata from cross-fading when a
    /// track change swaps the string while the lyrics layout is still animating.
    private var trackInfoRow: some View {
        let bar = state.playback.playerBar

        return HStack(spacing: 10) {
            Text(TimeFormat.elapsed(bar.positionMs))
                .font(.system(size: 12))
                .foregroundColor(.white.opacity(0.75))
                .frame(width: 48, alignment: .leading)
                .contentTransition(.identity)
                .transition(.identity)

            Text("\(bar.artist.isEmpty ? "Fluyer" : bar.artist) • \(bar.title.isEmpty ? "No track playing" : bar.title)")
                .font(.system(size: 15, weight: .medium))
                .foregroundColor(.white.opacity(0.9))
                .lineLimit(1)
                .truncationMode(.tail)
                .frame(maxWidth: .infinity)
                .transition(.identity)
                .contentTransition(.identity)

            Text(TimeFormat.elapsed(bar.durationMs))
                .font(.system(size: 12))
                .foregroundColor(.white.opacity(0.75))
                .frame(width: 48, alignment: .trailing)
                .contentTransition(.identity)
                .transition(.identity)
        }
        .monospacedDigit()
    }

    /// Mirrors `grid-cols-[1fr_auto_auto_auto_1fr]`: repeat hugs the transport group on
    /// its left, shuffle on its right, play/pause stays centred.
    private var transportControls: some View {
        HStack(spacing: 8) {
            Color.clear.frame(maxWidth: .infinity)

            iconButton(
                icon: PlaybackIcons.repeatIcon(state.playback.playerBar.repeatMode),
                size: 15,
                isActive: state.playback.playerBar.repeatMode != .none,
                help: "Repeat"
            ) {
                state.playback.cycleRepeat()
            }

            iconButton(icon: "backward.fill", size: 18, help: "Previous") {
                state.playback.previous()
            }

            iconButton(
                icon: state.playback.playerBar.isPlaying ? "pause.fill" : "play.fill",
                size: 24,
                help: state.playback.playerBar.isPlaying ? "Pause" : "Play"
            ) {
                state.playback.togglePlay()
            }

            iconButton(icon: "forward.fill", size: 18, help: "Next") {
                state.playback.next()
            }

            iconButton(
                icon: "shuffle",
                size: 15,
                isActive: state.playback.playerBar.isShuffled,
                help: "Shuffle"
            ) {
                state.playback.shuffle()
            }

            Color.clear.frame(maxWidth: .infinity)
        }
    }

    private var volumeRow: some View {
        let volume = state.playback.playerBar.volume

        return HStack(spacing: 12) {
            iconButton(
                icon: "speaker.slash.fill",
                size: 13,
                isActive: volume > 0.001,
                help: volume <= 0.001 ? "Unmute" : "Mute"
            ) {
                state.playback.toggleMute()
            }
            .frame(width: 20)

            SliderTrack(
                fraction: Binding(
                    get: { state.playback.playerBar.volume },
                    set: { state.playback.setVolume($0) }
                ),
                isThin: true
            )
            .frame(height: 14)

            iconButton(
                icon: PlaybackIcons.volume(volume),
                size: 13,
                isActive: volume > 0.001,
                help: "Max volume"
            ) {
                state.playback.setVolume(1.0)
            }
            .frame(width: 20)
        }
    }

    private func iconButton(
        icon: String,
        size: CGFloat,
        isActive: Bool = true,
        help: String,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Image(systemName: icon)
                .font(.system(size: size))
                .foregroundColor(.white.opacity(isActive ? 0.9 : 0.4))
                .frame(width: size + 18, height: size + 18)
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .help(help)
    }

    // MARK: - Lyrics Column

    private var lyricsColumn: some View {
        ScrollViewReader { proxy in
            ScrollView(.vertical) {
                LazyVStack(alignment: .leading, spacing: 28) {
                    ForEach(Array(state.playback.playView.lyrics.indices), id: \.self) { i in
                        lyricLine(i)
                    }
                }
                .padding(.vertical, 260)
                .padding(.horizontal, 28)
            }
            .scrollIndicators(.hidden)
            .mask(
                LinearGradient(
                    stops: [
                        .init(color: .clear, location: 0),
                        .init(color: .black, location: 0.4),
                        .init(color: .black, location: 0.6),
                        .init(color: .clear, location: 1),
                    ],
                    startPoint: .top,
                    endPoint: .bottom
                )
            )
            .onChange(of: activeLyricIndex) { _, newIndex in
                guard newIndex >= 0, newIndex < state.playback.playView.lyrics.count else { return }
                withAnimation(.spring(response: 0.45, dampingFraction: 0.8)) {
                    proxy.scrollTo(newIndex, anchor: .center)
                }
            }
        }
    }

    private func lyricLine(_ i: Int) -> some View {
        let line = state.playback.playView.lyrics[i]
        let isActive = (i == activeLyricIndex)

        return Group {
            if line.text.isEmpty {
                Image(systemName: "music.note")
                    .font(.system(size: isActive ? 33 : 27))
                    .foregroundColor(.white.opacity(isActive ? 0.95 : 0.5))
                    .frame(width: isActive ? 38 : 32, alignment: .leading)
            } else {
                Text(line.text)
                    .font(.system(size: isActive ? 30 : 25, weight: .bold))
                    .foregroundColor(.white.opacity(isActive ? 1 : 0.4))
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.vertical, 20)
        .scaleEffect(isActive ? 1.0 : 0.94, anchor: .leading)
        .animation(.spring(response: 0.3, dampingFraction: 0.7), value: isActive)
        .contentShape(Rectangle())
        .onTapGesture {
            state.playback.seek(toMs: line.timestampMs)
        }
        .id(i)
    }

    // MARK: - Back Button

    private var backButton: some View {
        Button(action: handleBack) {
            Image(systemName: "chevron.left")
                .font(.system(size: 13, weight: .semibold))
                .foregroundColor(.white)
                .frame(width: 28, height: 28)
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .help("Back (Esc)")
        .padding(.leading, 12)
        .padding(.top, 24)
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
                state.playback.showPlayView = false
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
