import SwiftUI
import FluyerCore

public struct PlayerBarView: View {
    @Bindable var state: AppState
    @State private var isHoveringProgress = false

    public var body: some View {
        VStack(spacing: 6) {
            // 1. Sleek top progress bar (5px height, hover and seek)
            GeometryReader { geo in
                ZStack(alignment: .leading) {
                    Rectangle()
                        .fill(Color.white.opacity(0.12))
                        .frame(height: isHoveringProgress ? 5 : 3)

                    Rectangle()
                        .fill(Color.white)
                        .frame(width: geo.size.width * CGFloat(state.playerBar.progressPct), height: isHoveringProgress ? 5 : 3)
                }
                .contentShape(Rectangle())
                .onHover { isHoveringProgress = $0 }
                .gesture(
                    DragGesture(minimumDistance: 0)
                        .onChanged { value in
                            let pct = Float(value.location.x / geo.size.width)
                            state.seekPercent(pct)
                        }
                )
            }
            .frame(height: 6)
            .padding(.horizontal, 16)

            // 2. Control Pill Container (rounded-full glass capsule matching wxWidgets PillPanel)
            HStack(spacing: 0) {
                // Column 1: Playback controls (Previous, Play/Pause, Next)
                HStack(spacing: 12) {
                    Button(action: { state.previous() }) {
                        Image(systemName: "backward.fill")
                            .font(.system(size: 15))
                            .foregroundColor(.white)
                            .frame(width: 32, height: 32)
                    }
                    .buttonStyle(.plain)

                    Button(action: { state.togglePlay() }) {
                        Image(systemName: state.playerBar.isPlaying ? "pause.fill" : "play.fill")
                            .font(.system(size: 17))
                            .foregroundColor(.white)
                            .frame(width: 32, height: 32)
                    }
                    .buttonStyle(.plain)

                    Button(action: { state.next() }) {
                        Image(systemName: "forward.fill")
                            .font(.system(size: 15))
                            .foregroundColor(.white)
                            .frame(width: 32, height: 32)
                    }
                    .buttonStyle(.plain)
                }
                .frame(width: 190, alignment: .leading)

                Spacer(minLength: 16)

                // Column 2: Track Info (Dynamically centered, hugs visual content)
                HStack(spacing: 10) {
                    // Cover artwork (Clickable -> opens PlayView!)
                    Button(action: {
                        withAnimation(.spring(response: 0.35, dampingFraction: 0.8)) {
                            state.showPlayView = true
                        }
                    }) {
                        CurrentCoverView(state: state, side: 40)
                    }
                    .buttonStyle(.plain)
                    .help("Open Now Playing")

                    VStack(alignment: .leading, spacing: 2) {
                        Text(state.playerBar.title.isEmpty ? "No track playing" : state.playerBar.title)
                            .font(.system(size: 13, weight: .medium))
                            .foregroundColor(.white)
                            .lineLimit(1)

                        Text(state.playerBar.artist.isEmpty ? "Fluyer" : state.playerBar.artist)
                            .font(.system(size: 11))
                            .foregroundColor(.white.opacity(0.6))
                            .lineLimit(1)
                    }
                }

                Spacer(minLength: 16)

                // Column 3: Secondary controls (Repeat, Shuffle, Volume)
                HStack(spacing: 10) {
                    Button(action: { state.cycleRepeat() }) {
                        Image(systemName: repeatIconName)
                            .font(.system(size: 13))
                            .foregroundColor(state.playerBar.repeatMode != .none ? .white : .white.opacity(0.5))
                            .frame(width: 24, height: 24)
                    }
                    .buttonStyle(.plain)

                    Button(action: { state.shuffle() }) {
                        Image(systemName: "shuffle")
                            .font(.system(size: 13))
                            .foregroundColor(state.playerBar.isShuffled ? .white : .white.opacity(0.5))
                            .frame(width: 24, height: 24)
                    }
                    .buttonStyle(.plain)

                    Button(action: { state.toggleMute() }) {
                        Image(systemName: volumeIconName)
                            .font(.system(size: 13))
                            .foregroundColor(state.playerBar.volume > 0.001 ? .white.opacity(0.85) : .white.opacity(0.4))
                            .frame(width: 24, height: 24)
                    }
                    .buttonStyle(.plain)
                    .help(state.playerBar.volume <= 0.001 ? "Unmute" : "Mute")

                    VolumeBar(volume: Binding(
                        get: { state.playerBar.volume },
                        set: { state.setVolume($0) }
                    ))
                    .frame(width: 80, height: 24)
                }
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
            .animation(.easeInOut(duration: 0.25), value: state.playerBar.title)
            .padding(.horizontal, 16)
            .padding(.bottom, 10)
        }
    }

    private var volumeIconName: String {
        let vol = state.playerBar.volume
        if vol <= 0.001 {
            return "speaker.slash.fill"
        } else if vol < 0.33 {
            return "speaker.wave.1.fill"
        } else if vol < 0.66 {
            return "speaker.wave.2.fill"
        } else {
            return "speaker.wave.3.fill"
        }
    }

    private var repeatIconName: String {
        switch state.playerBar.repeatMode {
        case .one: return "repeat.1"
        case .all, .none: return "repeat"
        }
    }
}

// MARK: - Volume Bar

private struct VolumeBar: View {
    @Binding var volume: Float
    @State private var isHovering = false

    var body: some View {
        GeometryReader { geo in
            let width = geo.size.width
            let progress = CGFloat(max(0.0, min(1.0, volume)))

            ZStack(alignment: .leading) {
                // Background track
                Capsule()
                    .fill(Color.white.opacity(0.18))
                    .frame(height: isHovering ? 5 : 4)

                // Fill track
                Capsule()
                    .fill(Color.white.opacity(isHovering ? 1.0 : 0.85))
                    .frame(width: max(0, width * progress), height: isHovering ? 5 : 4)
            }
            .frame(maxHeight: .infinity, alignment: .center)
            .contentShape(Rectangle())
            .onHover { isHovering = $0 }
            .gesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { value in
                        let pct = Float(value.location.x / width)
                        volume = max(0.0, min(1.0, pct))
                    }
            )
        }
    }
}
