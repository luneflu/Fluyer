import FluyerCore
import SwiftUI

/// Now-playing surface mirroring `Fluyer/src/routes/play/+page.svelte`.
///
/// ponytail: the Svelte page is a two-column grid — cover art (col 1, row 1), the
/// control card (col 1, row 2) and the lyrics scroller spanning both rows (col 2).
/// Without lyrics the page collapses to a single centered column. The proportions
/// below mirror `md:grid-cols-[40%_55%]` / `md:grid-cols-[50%]`.
public struct PlayView: View {
    @Bindable var state: AppState

    @State private var isIdle = false
    @State private var backButtonHidden = false
    @State private var idleTask: Task<Void, Never>?

    /// Svelte hides the control-card column entirely below a single lyric line.
    private var hasLyrics: Bool { state.playView.lyrics.count > 1 }

    private var activeLyricIndex: Int { Int(state.playView.currentLyricIndex) }

    public var body: some View {
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
                switch phase {
                case .active:
                    resetIdleTimer()
                case .ended:
                    break
                }
            }
            .onExitCommand {
                handleBackWithDelay()
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

            ProgressTrack(
                progress: Binding(
                    get: { state.playerBar.progressPct },
                    set: { state.seekPercent($0) }
                )
            )
            .frame(height: 8)

            transportControls
                .padding(.top, 4)

            volumeRow
        }
        // .padding(20)
        // .background(
        //     RoundedRectangle(cornerRadius: 8)
        //         .fill(Color.white.opacity(0.06))
        //         .overlay(
        //             RoundedRectangle(cornerRadius: 8)
        //                 .stroke(Color.white.opacity(0.12), lineWidth: 1)
        //         )
        // )
        .fixedSize(horizontal: false, vertical: true)
    }

    /// Elapsed time on the left, `artist • title` centred, total duration right.
    ///
    /// ponytail: `.transition(.identity)` keeps the metadata from cross-fading when a
    /// track change swaps the string while the lyrics layout is still animating.
    private var trackInfoRow: some View {
        HStack(spacing: 10) {
            Text(formatTime(state.playerBar.positionMs))
                .font(.system(size: 12))
                .foregroundColor(.white.opacity(0.75))
                .frame(width: 48, alignment: .leading)
                .contentTransition(.identity)
                .transition(.identity)

            Text(
                "\(state.playerBar.artist.isEmpty ? "Fluyer" : state.playerBar.artist) • \(state.playerBar.title.isEmpty ? "No track playing" : state.playerBar.title)"
            )
            .font(.system(size: 15, weight: .medium))
            .foregroundColor(.white.opacity(0.9))
            .lineLimit(1)
            .truncationMode(.tail)
            .frame(maxWidth: .infinity)
            .transition(.identity)
            .contentTransition(.identity)

            Text(formatTime(state.playerBar.durationMs))
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
                icon: repeatIconName,
                size: 15,
                isActive: state.playerBar.repeatMode != .none,
                help: "Repeat",
                action: { state.cycleRepeat() }
            )

            iconButton(
                icon: "backward.fill", size: 18, help: "Previous", action: { state.previous() })

            iconButton(
                icon: state.playerBar.isPlaying ? "pause.fill" : "play.fill",
                size: 24,
                help: state.playerBar.isPlaying ? "Pause" : "Play",
                action: { state.togglePlay() }
            )

            iconButton(icon: "forward.fill", size: 18, help: "Next", action: { state.next() })

            iconButton(
                icon: "shuffle",
                size: 15,
                isActive: state.playerBar.isShuffled,
                help: "Shuffle",
                action: { state.shuffle() }
            )

            Color.clear.frame(maxWidth: .infinity)
        }
    }

    private var volumeRow: some View {
        HStack(spacing: 12) {
            iconButton(
                icon: "speaker.slash.fill",
                size: 13,
                isActive: state.playerBar.volume > 0.001,
                help: state.playerBar.volume <= 0.001 ? "Unmute" : "Mute",
                action: { state.toggleMute() }
            )
            .frame(width: 20)

            ProgressTrack(
                progress: Binding(
                    get: { state.playerBar.volume },
                    set: { state.setVolume($0) }
                ), isThin: true
            )
            .frame(height: 14)

            iconButton(
                icon: volumeIconName,
                size: 13,
                isActive: state.playerBar.volume > 0.001,
                help: "Max volume",
                action: { state.setVolume(1.0) }
            )
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
                    ForEach(Array(state.playView.lyrics.indices), id: \.self) { i in
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
                guard newIndex >= 0, newIndex < state.playView.lyrics.count else { return }
                withAnimation(.spring(response: 0.45, dampingFraction: 0.8)) {
                    proxy.scrollTo(newIndex, anchor: .center)
                }
            }
        }
    }

    private func lyricLine(_ i: Int) -> some View {
        let line = state.playView.lyrics[i]
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
            state.seek(positionMs: line.timestampMs)
        }
        .id(i)
    }

    // MARK: - Back Button

    private var backButton: some View {
        Button(action: { handleBackWithDelay() }) {
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

    private func handleBackWithDelay() {
        guard !backButtonHidden else { return }
        backButtonHidden = true
        idleTask?.cancel()
        Task { @MainActor in
            try? await Task.sleep(nanoseconds: 300_000_000)
            withAnimation(.spring(response: 0.35, dampingFraction: 0.8)) {
                state.showPlayView = false
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

    // MARK: - Helpers

    private var repeatIconName: String {
        switch state.playerBar.repeatMode {
        case .one: return "repeat.1"
        case .all, .none: return "repeat"
        }
    }

    private var volumeIconName: String {
        let vol = state.playerBar.volume
        if vol <= 0.001 { return "speaker.slash.fill" }
        if vol < 0.33 { return "speaker.wave.1.fill" }
        if vol < 0.66 { return "speaker.wave.2.fill" }
        return "speaker.wave.3.fill"
    }

    private func formatTime(_ ms: UInt64) -> String {
        let totalSec = ms / 1000
        let min = totalSec / 60
        let sec = totalSec % 60
        return String(format: "%d:%02d", min, sec)
    }
}

// MARK: - Progress Track

/// Draggable progress bar matching the Svelte `ProgressBar` (size `md` / `sm`).
private struct ProgressTrack: View {
    @Binding var progress: Float
    var isThin: Bool = false

    @State private var isHovering = false

    private var thickness: CGFloat {
        if isThin { return isHovering ? 5 : 4 }
        return isHovering ? 6 : 5
    }

    var body: some View {
        GeometryReader { geo in
            let width = geo.size.width
            let pct = CGFloat(max(0.0, min(1.0, progress)))

            ZStack(alignment: .leading) {
                Capsule()
                    .fill(Color.white.opacity(0.18))
                    .frame(height: thickness)

                Capsule()
                    .fill(Color.white.opacity(isHovering ? 1.0 : 0.9))
                    .frame(width: max(0, width * pct), height: thickness)
            }
            .frame(maxHeight: .infinity, alignment: .center)
            .contentShape(Rectangle())
            .onHover { isHovering = $0 }
            .gesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { value in
                        progress = Float(value.location.x / width)
                    }
            )
        }
    }
}
