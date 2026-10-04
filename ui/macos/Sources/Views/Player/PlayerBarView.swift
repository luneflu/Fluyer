import SwiftUI
import FluyerCore

/// Always-visible transport strip at the bottom of the library window: seek bar,
/// transport buttons, now-playing summary and secondary controls.
struct PlayerBarView: View {
    @Bindable var state: AppState
    @State private var isHoveringProgress = false

    var body: some View {
        VStack(spacing: 6) {
            seekBar
            controls
        }
    }

    private var seekBar: some View {
        GeometryReader { geo in
            ZStack(alignment: .leading) {
                Rectangle()
                    .fill(Color.white.opacity(0.12))
                    .frame(height: isHoveringProgress ? 5 : 3)

                Rectangle()
                    .fill(Color.white)
                    .frame(
                        width: geo.size.width * CGFloat(state.playback.playerBar.progressPct.clamped(to: 0...1)),
                        height: isHoveringProgress ? 5 : 3
                    )
            }
            .contentShape(Rectangle())
            .onHover { isHoveringProgress = $0 }
            .gesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { value in
                        guard geo.size.width > 0 else { return }
                        state.playback.seek(toFraction: Float(value.location.x / geo.size.width))
                    }
            )
        }
        .frame(height: 6)
        .padding(.horizontal, 16)
    }

    private var controls: some View {
        HStack(spacing: 0) {
            transport
                .frame(width: 190, alignment: .leading)

            Spacer(minLength: 16)

            nowPlaying

            Spacer(minLength: 16)

            secondaryControls
                .frame(width: 190, alignment: .trailing)
        }
        .padding(.horizontal, 18)
        .frame(height: 52)
        .background(
            Capsule()
                .fill(Color.white.opacity(0.08))
                .overlay(
                    Capsule()
                        .stroke(Color.white.opacity(0.18), lineWidth: 1)
                )
        )
        .animation(.easeInOut(duration: 0.25), value: state.playback.playerBar.title)
        .padding(.horizontal, 16)
        .padding(.bottom, 10)
    }

    private var transport: some View {
        HStack(spacing: 12) {
            iconButton("backward.fill", size: 15, help: "Previous") {
                state.playback.previous()
            }

            iconButton(
                state.playback.playerBar.isPlaying ? "pause.fill" : "play.fill",
                size: 17,
                help: state.playback.playerBar.isPlaying ? "Pause" : "Play"
            ) {
                state.playback.togglePlay()
            }

            iconButton("forward.fill", size: 15, help: "Next") {
                state.playback.next()
            }
        }
    }

    private var nowPlaying: some View {
        HStack(spacing: 10) {
            Button(action: {
                withAnimation(.spring(response: 0.35, dampingFraction: 0.8)) {
                    state.playback.showPlayView = true
                }
            }) {
                CurrentCoverView(state: state, side: 40)
            }
            .buttonStyle(.plain)
            .help("Open Now Playing")

            VStack(alignment: .leading, spacing: 2) {
                Text(state.playback.playerBar.title.isEmpty ? "No track playing" : state.playback.playerBar.title)
                    .font(.system(size: 13, weight: .medium))
                    .foregroundColor(.white)
                    .lineLimit(1)

                Text(state.playback.playerBar.artist.isEmpty ? "Fluyer" : state.playback.playerBar.artist)
                    .font(.system(size: 11))
                    .foregroundColor(.white.opacity(0.6))
                    .lineLimit(1)
            }
        }
    }

    private var secondaryControls: some View {
        let volume = state.playback.playerBar.volume

        return HStack(spacing: 10) {
            iconButton(
                PlaybackIcons.repeatIcon(state.playback.playerBar.repeatMode),
                size: 13,
                help: "Repeat",
                isActive: state.playback.playerBar.repeatMode != .none
            ) {
                state.playback.cycleRepeat()
            }

            iconButton(
                "shuffle",
                size: 13,
                help: "Shuffle",
                isActive: state.playback.playerBar.isShuffled
            ) {
                state.playback.shuffle()
            }

            iconButton(
                PlaybackIcons.volume(volume),
                size: 13,
                help: volume <= 0.001 ? "Unmute" : "Mute",
                isActive: volume > 0.001
            ) {
                state.playback.toggleMute()
            }

            SliderTrack(
                fraction: Binding(
                    get: { state.playback.playerBar.volume },
                    set: { state.playback.setVolume($0) }
                ),
                isThin: true
            )
            .frame(width: 80, height: 24)
        }
    }

    private func iconButton(
        _ icon: String,
        size: CGFloat,
        help: String,
        isActive: Bool = true,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Image(systemName: icon)
                .font(.system(size: size))
                .foregroundColor(isActive ? .white : .white.opacity(0.5))
                .frame(width: 24, height: 24)
        }
        .buttonStyle(.plain)
        .help(help)
    }
}
