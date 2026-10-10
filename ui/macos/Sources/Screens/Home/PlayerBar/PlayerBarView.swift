import SwiftUI
import FluyerCore

/// Always-visible transport strip at the bottom of the library window: seek bar,
/// transport buttons, now-playing summary and secondary controls.
struct PlayerBarView: View {
    @Bindable var playback: PlaybackState
    let covers: CoverState
    @State private var isHoveringProgress = false

    var body: some View {
        VStack(spacing: Layout.PlayerBar.seekToControls) {
            seekBar
            controls
        }
    }

    private var seekBar: some View {
        GeometryReader { geo in
            ZStack(alignment: .leading) {
                Rectangle()
                    .fill(Color.white.opacity(0.12))
                    .frame(height: isHoveringProgress ? Layout.PlayerBar.seekThicknessHovered : Layout.PlayerBar.seekThickness)

                Rectangle()
                    .fill(Color.white)
                    .frame(
                        width: geo.size.width * CGFloat(playback.playerBar.progressPct.clamped(to: 0...1)),
                        height: isHoveringProgress ? Layout.PlayerBar.seekThicknessHovered : Layout.PlayerBar.seekThickness
                    )
            }
            .contentShape(Rectangle())
            .onHover { isHoveringProgress = $0 }
            .gesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { value in
                        guard geo.size.width > 0 else { return }
                        playback.seek(toFraction: Float(value.location.x / geo.size.width))
                    }
            )
        }
        .frame(height: Layout.PlayerBar.seekHitHeight)
        .padding(.horizontal, Layout.PlayerBar.seekHorizontalPadding)
    }

    private var controls: some View {
        HStack(spacing: 0) {
            transport
                .frame(width: Layout.PlayerBar.sideColumnWidth, alignment: .leading)

            Spacer(minLength: Layout.PlayerBar.minGapToCenter)

            nowPlaying

            Spacer(minLength: Layout.PlayerBar.minGapToCenter)

            secondaryControls
                .frame(width: Layout.PlayerBar.sideColumnWidth, alignment: .trailing)
        }
        .padding(.horizontal, Layout.PlayerBar.controlsHorizontalPadding)
        .frame(height: Layout.PlayerBar.controlsHeight)
        .background(
            Capsule()
                .fill(Color.white.opacity(0.08))
                .overlay(
                    Capsule()
                        .stroke(Color.white.opacity(0.18), lineWidth: 1)
                )
        )
        .animation(.easeInOut(duration: 0.25), value: playback.playerBar.title)
        .padding(.horizontal, Layout.PlayerBar.horizontalPadding)
        .padding(.bottom, Layout.PlayerBar.bottomPadding)
    }

    private var transport: some View {
        HStack(spacing: Layout.PlayerBar.transportSpacing) {
            iconButton("backward.fill", size: 15, help: "Previous") {
                playback.previous()
            }

            iconButton(
                playback.playerBar.isPlaying ? "pause.fill" : "play.fill",
                size: 17,
                help: playback.playerBar.isPlaying ? "Pause" : "Play"
            ) {
                playback.togglePlay()
            }

            iconButton("forward.fill", size: 15, help: "Next") {
                playback.next()
            }
        }
    }

    private var nowPlaying: some View {
        HStack(spacing: Layout.PlayerBar.coverToText) {
            Button(action: {
                withAnimation(.spring(response: 0.35, dampingFraction: 0.8)) {
                    playback.showPlayView = true
                }
            }) {
                CurrentCoverView(playback: playback, covers: covers, side: Layout.PlayerBar.coverSide)
            }
            .buttonStyle(.plain)
            .help("Open Now Playing")

            VStack(alignment: .leading, spacing: Layout.PlayerBar.titleToArtist) {
                Text(playback.playerBar.title.isEmpty ? "No track playing" : playback.playerBar.title)
                    .font(.system(size: 13, weight: .medium))
                    .foregroundColor(.white)
                    .lineLimit(1)

                Text(playback.playerBar.artist.isEmpty ? "Fluyer" : playback.playerBar.artist)
                    .font(.system(size: 11))
                    .foregroundColor(.white.opacity(0.6))
                    .lineLimit(1)
            }
        }
    }

    private var secondaryControls: some View {
        let volume = playback.playerBar.volume

        return HStack(spacing: Layout.PlayerBar.secondarySpacing) {
            iconButton(
                PlaybackIcons.repeatIcon(playback.playerBar.repeatMode),
                size: 13,
                help: "Repeat",
                isActive: playback.playerBar.repeatMode != .none
            ) {
                playback.cycleRepeat()
            }

            iconButton(
                "shuffle",
                size: 13,
                help: "Shuffle",
                isActive: playback.playerBar.isShuffled
            ) {
                playback.shuffle()
            }

            iconButton(
                PlaybackIcons.volume(volume),
                size: 13,
                help: volume <= 0.001 ? "Unmute" : "Mute",
                isActive: volume > 0.001
            ) {
                playback.toggleMute()
            }

            SliderTrack(
                fraction: Binding(
                    get: { playback.playerBar.volume },
                    set: { playback.setVolume($0) }
                ),
                isThin: true
            )
            .frame(width: Layout.PlayerBar.volumeSliderWidth, height: Layout.PlayerBar.volumeSliderHeight)
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
                .frame(width: Layout.PlayerBar.iconButtonSide, height: Layout.PlayerBar.iconButtonSide)
        }
        .buttonStyle(.plain)
        .help(help)
    }
}
