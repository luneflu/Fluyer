package org.alvindimas05.fluyer.screens.home.playerbar

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Pause
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.SkipNext
import androidx.compose.material.icons.rounded.SkipPrevious
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.alvindimas05.fluyer.components.CurrentCover
import org.alvindimas05.fluyer.state.AppState

/** Bottom bar: cover, title/artist, prev/play/next, progress line. Tap opens the play screen. */
@Composable
internal fun PlayerBar(state: AppState) {
    val bar = state.playback.bar
    if (bar.trackIndex < 0) return
    val clock = state.playback.clock
    Column(Modifier.fillMaxWidth().background(Color.Black.copy(alpha = 0.55f)).clickable { state.playback.openPlayView(true) }) {
        Row(Modifier.padding(horizontal = 12.dp, vertical = 6.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            CurrentCover(state, 96, Modifier.size(44.dp))
            Column(Modifier.weight(1f)) {
                Text(bar.title, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text(bar.artist, maxLines = 1, overflow = TextOverflow.Ellipsis, fontSize = 12.sp, color = Color.White.copy(alpha = 0.6f))
            }
            IconButton({ state.playback.previous() }) { Icon(Icons.Rounded.SkipPrevious, "Previous") }
            IconButton({ state.playback.togglePlay() }) {
                Icon(if (bar.isPlaying) Icons.Rounded.Pause else Icons.Rounded.PlayArrow, if (bar.isPlaying) "Pause" else "Play", Modifier.size(32.dp))
            }
            IconButton({ state.playback.next() }) { Icon(Icons.Rounded.SkipNext, "Next") }
        }
        LinearProgressIndicator({ clock.progressPct.coerceIn(0f, 1f) }, Modifier.fillMaxWidth().height(2.dp), color = Color.White, trackColor = Color.White.copy(alpha = 0.2f))
    }
}
