package org.alvindimas05.fluyer.screens.home.queue

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.KeyboardArrowDown
import androidx.compose.material.icons.rounded.KeyboardArrowUp
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.alvindimas05.fluyer.components.Cover
import org.alvindimas05.fluyer.state.AppState
import org.alvindimas05.fluyer.support.ThumbnailKey
import uniffi.fluyer_core.TrackItemViewModel

/** One queue row: cover, title/artist, move up / down, remove. */
@Composable
internal fun QueueRow(state: AppState, index: Int, track: TrackItemViewModel) {
    val q = state.queue
    Row(Modifier.fillMaxWidth().clickable { q.goto(index) }.padding(start = 16.dp, end = 4.dp, top = 4.dp, bottom = 4.dp), verticalAlignment = Alignment.CenterVertically) {
        Cover(state, ThumbnailKey.track(track.index, 88), 88, Modifier.size(40.dp)) { state.engine?.artworkLoadTrackThumbnail(track.index, it) }
        Column(Modifier.weight(1f).padding(horizontal = 12.dp)) {
            Text(track.title, maxLines = 1, overflow = TextOverflow.Ellipsis, fontWeight = if (track.isCurrent) FontWeight.Bold else null)
            Text(track.artist, maxLines = 1, overflow = TextOverflow.Ellipsis, fontSize = 12.sp, color = Color.White.copy(alpha = 0.6f))
        }
        IconButton({ q.move(index, -1) }, enabled = index > 0) { Icon(Icons.Rounded.KeyboardArrowUp, "Move up") }
        IconButton({ q.move(index, 1) }, enabled = index < q.tracks.lastIndex) { Icon(Icons.Rounded.KeyboardArrowDown, "Move down") }
        IconButton({ q.remove(index) }) { Icon(Icons.Rounded.Close, "Remove") }
    }
}
