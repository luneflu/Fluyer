import SwiftUI
import FluyerCore

/// Control card under the cover on the play screen: elapsed / title / duration,
/// seek slider, transport buttons, volume row.
struct PlayControlsView: View {
    @Bindable var playback: PlaybackState

    var body: some View {
        VStack(spacing: 0) {
            trackInfoRow
                .padding(.top, Layout.PlayControls.infoTopPadding)
                .padding(.bottom, Layout.PlayControls.infoBottomPadding)

            SliderTrack(
                fraction: Binding(
                    get: { playback.playerBar.progressPct },
                    set: { playback.seek(toFraction: $0) }
                )
            )
            .frame(height: Layout.PlayControls.seekHeight)

            transportControls
                .padding(.top, Layout.PlayControls.transportTopPadding)

            volumeRow
        }
        .fixedSize(horizontal: false, vertical: true)
    }

    /// Elapsed time on the left, `artist • title` centred, total duration right.
    ///
    /// ponytail: `.transition(.identity)` keeps the metadata from cross-fading when a
    /// track change swaps the string while the lyrics layout is still animating.
    private var trackInfoRow: some View {
        let bar = playback.playerBar

        return HStack(spacing: Layout.PlayControls.infoSpacing) {
            Text(TimeFormat.elapsed(bar.positionMs))
                .font(.system(size: 12))
                .foregroundColor(.white.opacity(0.75))
                .frame(width: Layout.PlayControls.timeLabelWidth, alignment: .leading)
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
                .frame(width: Layout.PlayControls.timeLabelWidth, alignment: .trailing)
                .contentTransition(.identity)
                .transition(.identity)
        }
        .monospacedDigit()
    }

    /// Mirrors `grid-cols-[1fr_auto_auto_auto_1fr]`: repeat hugs the transport group on
    /// its left, shuffle on its right, play/pause stays centred.
    private var transportControls: some View {
        HStack(spacing: Layout.PlayControls.transportSpacing) {
            Color.clear.frame(maxWidth: .infinity)

            iconButton(
                icon: PlaybackIcons.repeatIcon(playback.playerBar.repeatMode),
                size: 15,
                isActive: playback.playerBar.repeatMode != .none,
                help: "Repeat"
            ) {
                playback.cycleRepeat()
            }

            iconButton(icon: "backward.fill", size: 18, help: "Previous") {
                playback.previous()
            }

            iconButton(
                icon: playback.playerBar.isPlaying ? "pause.fill" : "play.fill",
                size: 24,
                help: playback.playerBar.isPlaying ? "Pause" : "Play"
            ) {
                playback.togglePlay()
            }

            iconButton(icon: "forward.fill", size: 18, help: "Next") {
                playback.next()
            }

            iconButton(
                icon: "shuffle",
                size: 15,
                isActive: playback.playerBar.isShuffled,
                help: "Shuffle"
            ) {
                playback.shuffle()
            }

            Color.clear.frame(maxWidth: .infinity)
        }
    }

    private var volumeRow: some View {
        let volume = playback.playerBar.volume

        return HStack(spacing: Layout.PlayControls.volumeSpacing) {
            iconButton(
                icon: "speaker.slash.fill",
                size: 13,
                isActive: volume > 0.001,
                help: volume <= 0.001 ? "Unmute" : "Mute"
            ) {
                playback.toggleMute()
            }
            .frame(width: Layout.PlayControls.volumeIconWidth)

            SliderTrack(
                fraction: Binding(
                    get: { playback.playerBar.volume },
                    set: { playback.setVolume($0) }
                ),
                isThin: true
            )
            .frame(height: Layout.PlayControls.volumeSliderHeight)

            iconButton(
                icon: PlaybackIcons.volume(volume),
                size: 13,
                isActive: volume > 0.001,
                help: "Max volume"
            ) {
                playback.setVolume(1.0)
            }
            .frame(width: Layout.PlayControls.volumeIconWidth)
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
                .frame(width: size + Layout.PlayControls.iconButtonPadding, height: size + Layout.PlayControls.iconButtonPadding)
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .help(help)
    }
}
