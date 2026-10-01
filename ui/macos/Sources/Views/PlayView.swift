import SwiftUI
import FluyerCore

public struct PlayView: View {
    @Bindable var state: AppState

    public var body: some View {
        GeometryReader { geo in
            ZStack {
                VStack(spacing: 0) {
                    // Top: Back Button matching wxWidgets ("Back" button)
                    HStack {
                        Button(action: {
                            withAnimation(.spring(response: 0.35, dampingFraction: 0.8)) {
                                state.showPlayView = false
                            }
                        }) {
                            HStack(spacing: 6) {
                                Image(systemName: "chevron.left")
                                    .font(.system(size: 13, weight: .semibold))
                                Text("Back")
                                    .font(.system(size: 13))
                            }
                            .foregroundColor(.white.opacity(0.85))
                            .padding(.horizontal, 14)
                            .padding(.vertical, 8)
                            .background(Color.white.opacity(0.12))
                            .clipShape(Capsule())
                        }
                        .buttonStyle(.plain)

                        Spacer()
                    }
                    .padding(.horizontal, 32)
                    .padding(.top, 20)

                    // Center Content: Left Control Card + Right Lyrics Card
                    HStack(alignment: .center, spacing: 32) {
                        // Left: Glass Control Card matching wxWidgets ControlCardPanel
                        VStack(spacing: 20) {
                            // Cover art with rounded corners
                            CurrentCoverView(state: state, side: 290)
                                .overlay(
                                    RoundedRectangle(cornerRadius: 12)
                                        .stroke(Color.white.opacity(0.15), lineWidth: 1)
                                )
                                .shadow(color: .black.opacity(0.35), radius: 16, x: 0, y: 8)

                            // Track Metadata
                            VStack(alignment: .leading, spacing: 4) {
                                Text(state.playerBar.title.isEmpty ? "No track playing" : state.playerBar.title)
                                    .font(.system(size: 20, weight: .bold))
                                    .foregroundColor(.white)
                                    .lineLimit(1)

                                Text(state.playerBar.artist.isEmpty ? "Fluyer" : state.playerBar.artist)
                                    .font(.system(size: 14, weight: .medium))
                                    .foregroundColor(.white.opacity(0.7))
                                    .lineLimit(1)

                                if !state.playerBar.album.isEmpty {
                                    Text(state.playerBar.album)
                                        .font(.system(size: 12))
                                        .foregroundColor(.white.opacity(0.45))
                                        .lineLimit(1)
                                }
                            }
                            .frame(maxWidth: 290, alignment: .leading)

                            // Scrubber & Time
                            VStack(spacing: 6) {
                                GeometryReader { geo in
                                    ZStack(alignment: .leading) {
                                        Capsule()
                                            .fill(Color.white.opacity(0.15))
                                            .frame(height: 5)

                                        Capsule()
                                            .fill(Color.white)
                                            .frame(width: geo.size.width * CGFloat(state.playerBar.progressPct), height: 5)
                                    }
                                    .contentShape(Rectangle())
                                    .gesture(
                                        DragGesture(minimumDistance: 0)
                                            .onChanged { value in
                                                let pct = Float(value.location.x / geo.size.width)
                                                state.seekPercent(pct)
                                            }
                                    )
                                }
                                .frame(height: 5)

                                HStack {
                                    Text(formatTime(state.playerBar.positionMs))
                                        .font(.system(size: 11, design: .monospaced))
                                        .foregroundColor(.white.opacity(0.5))
                                    Spacer()
                                    Text(formatTime(state.playerBar.durationMs))
                                        .font(.system(size: 11, design: .monospaced))
                                        .foregroundColor(.white.opacity(0.5))
                                }
                            }
                            .frame(width: 290)

                            // Controls Capsule
                            HStack(spacing: 24) {
                                Button(action: { state.previous() }) {
                                    Image(systemName: "backward.fill")
                                        .font(.system(size: 18))
                                        .foregroundColor(.white)
                                }
                                .buttonStyle(.plain)

                                Button(action: { state.togglePlay() }) {
                                    Image(systemName: state.playerBar.isPlaying ? "pause.fill" : "play.fill")
                                        .font(.system(size: 24))
                                        .foregroundColor(.white)
                                }
                                .buttonStyle(.plain)

                                Button(action: { state.next() }) {
                                    Image(systemName: "forward.fill")
                                        .font(.system(size: 18))
                                        .foregroundColor(.white)
                                }
                                .buttonStyle(.plain)

                                Button(action: { state.shuffle() }) {
                                    Image(systemName: "shuffle")
                                        .font(.system(size: 15))
                                        .foregroundColor(state.playerBar.isShuffled ? .white : .white.opacity(0.4))
                                }
                                .buttonStyle(.plain)

                                Button(action: { state.cycleRepeat() }) {
                                    Image(systemName: repeatIconName)
                                        .font(.system(size: 15))
                                        .foregroundColor(state.playerBar.repeatMode != .none ? .white : .white.opacity(0.4))
                                }
                                .buttonStyle(.plain)
                            }
                            .padding(.horizontal, 20)
                            .padding(.vertical, 10)
                            .background(
                                Capsule()
                                    .fill(Color.white.opacity(0.08))
                                    .overlay(
                                        Capsule()
                                            .stroke(Color.white.opacity(0.15), lineWidth: 1)
                                    )
                            )
                        }
                        .padding(24)
                        .background(
                            RoundedRectangle(cornerRadius: 16)
                                .fill(Color.white.opacity(0.06))
                                .overlay(
                                    RoundedRectangle(cornerRadius: 16)
                                        .stroke(Color.white.opacity(0.12), lineWidth: 1)
                                )
                        )

                        // Right: Glass Lyrics Card matching wxWidgets LyricsCtrl
                        lyricsCard
                            .frame(maxWidth: .infinity, maxHeight: .infinity)
                    }
                    .padding(.horizontal, 32)
                    .padding(.vertical, 20)
            }
        }
        }
    }

    private var lyricsCard: some View {
        let lyrics = state.playView.lyrics
        let activeIndex = Int(state.playView.currentLyricIndex)

        return ZStack {
            RoundedRectangle(cornerRadius: 16)
                .fill(Color.white.opacity(0.06))
                .overlay(
                    RoundedRectangle(cornerRadius: 16)
                        .stroke(Color.white.opacity(0.12), lineWidth: 1)
                )

            if lyrics.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "music.mic")
                        .font(.system(size: 32))
                        .foregroundColor(.white.opacity(0.2))
                    Text("No lyrics found")
                        .font(.system(size: 15))
                        .foregroundColor(.white.opacity(0.4))
                }
            } else {
                ScrollViewReader { proxy in
                    ScrollView(showsIndicators: false) {
                        LazyVStack(alignment: .leading, spacing: 22) {
                            ForEach(0..<lyrics.count, id: \.self) { i in
                                let line = lyrics[i]
                                let isActive = (i == activeIndex)

                                Text(line.text.isEmpty ? "♪" : line.text)
                                    .font(.system(size: isActive ? 22 : 16, weight: isActive ? .bold : .regular))
                                    .foregroundColor(isActive ? .white : .white.opacity(0.38))
                                    .scaleEffect(isActive ? 1.02 : 1.0, anchor: .leading)
                                    .animation(.spring(response: 0.3, dampingFraction: 0.7), value: isActive)
                                    .contentShape(Rectangle())
                                    .onTapGesture {
                                        state.seek(positionMs: line.timestampMs)
                                    }
                                    .id(i)
                            }
                        }
                        .padding(.horizontal, 32)
                        .padding(.vertical, 140)
                    }
                    .onChange(of: activeIndex) { _, newIndex in
                        if newIndex >= 0 && newIndex < lyrics.count {
                            withAnimation(.spring(response: 0.45, dampingFraction: 0.8)) {
                                proxy.scrollTo(newIndex, anchor: .center)
                            }
                        }
                    }
                }
            }
        }
    }

    private var repeatIconName: String {
        switch state.playerBar.repeatMode {
        case .one: return "repeat.1"
        case .all, .none: return "repeat"
        }
    }

    private func formatTime(_ ms: UInt64) -> String {
        let totalSec = ms / 1000
        let min = totalSec / 60
        let sec = totalSec % 60
        return String(format: "%d:%02d", min, sec)
    }
}
