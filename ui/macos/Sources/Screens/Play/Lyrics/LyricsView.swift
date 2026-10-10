import SwiftUI
import FluyerCore

/// Scrolling, auto-centred lyrics column on the play screen. Tap a line to seek.
struct LyricsView: View {
    @Bindable var playback: PlaybackState

    private var activeLyricIndex: Int { playback.clock.currentLyricIndex }

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView(.vertical) {
                LazyVStack(alignment: .leading, spacing: Layout.Lyrics.lineSpacing) {
                    ForEach(Array(playback.playView.lyrics.indices), id: \.self) { i in
                        lyricLine(i)
                    }
                }
                .padding(.vertical, Layout.Lyrics.verticalInset)
                .padding(.horizontal, Layout.Lyrics.horizontalPadding)
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
                guard newIndex >= 0, newIndex < playback.playView.lyrics.count else { return }
                withAnimation(.spring(response: 0.45, dampingFraction: 0.8)) {
                    proxy.scrollTo(newIndex, anchor: .center)
                }
            }
        }
    }

    private func lyricLine(_ i: Int) -> some View {
        let line = playback.playView.lyrics[i]
        let isActive = (i == activeLyricIndex)

        return Group {
            if line.text.isEmpty {
                Image(systemName: "music.note")
                    .font(.system(size: isActive ? 33 : 27))
                    .foregroundColor(.white.opacity(isActive ? 0.95 : 0.5))
                    .frame(width: isActive ? Layout.Lyrics.noteWidthActive : Layout.Lyrics.noteWidth, alignment: .leading)
            } else {
                Text(line.text)
                    .font(.system(size: isActive ? 30 : 25, weight: .bold))
                    .foregroundColor(.white.opacity(isActive ? 1 : 0.4))
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.vertical, Layout.Lyrics.lineVerticalPadding)
        .scaleEffect(isActive ? 1.0 : 0.94, anchor: .leading)
        .animation(.spring(response: 0.3, dampingFraction: 0.7), value: isActive)
        .contentShape(Rectangle())
        .onTapGesture {
            playback.seek(toMs: line.timestampMs)
        }
        .id(i)
    }
}
