package org.alvindimas05.fluyer.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.alvindimas05.fluyer.state.AppState
import org.alvindimas05.fluyer.state.ThumbnailKey

/** Queue list: tap = goto, per-row remove / move up / move down, header clear. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun QueueSheet(state: AppState) {
    if (!state.queue.isOpen) return
    val q = state.queue
    ModalBottomSheet({ q.openSheet(false) }, sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true), containerColor = Color(0xFF1C1C1E)) {
        Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("Queue", Modifier.weight(1f), style = MaterialTheme.typography.titleLarge)
            TextButton({ q.clear() }, enabled = q.tracks.isNotEmpty()) { Text("Clear") }
        }
        if (q.tracks.isEmpty()) {
            Text("Queue is empty", Modifier.padding(24.dp), color = Color.White.copy(alpha = 0.6f))
        }
        LazyColumn(Modifier.navigationBarsPadding()) {
            itemsIndexed(q.tracks, key = { i, t -> "$i-${t.path}" }) { i, t ->
                Row(Modifier.fillMaxWidth().clickable { q.goto(i) }.padding(start = 16.dp, end = 4.dp, top = 4.dp, bottom = 4.dp), verticalAlignment = Alignment.CenterVertically) {
                    Cover(state, ThumbnailKey.track(t.index, 88), 88, Modifier.size(40.dp)) { state.engine?.loadTrackThumbnail(t.index, it) }
                    Column(Modifier.weight(1f).padding(horizontal = 12.dp)) {
                        Text(t.title, maxLines = 1, overflow = TextOverflow.Ellipsis, fontWeight = if (t.isCurrent) FontWeight.Bold else null)
                        Text(t.artist, maxLines = 1, overflow = TextOverflow.Ellipsis, fontSize = 12.sp, color = Color.White.copy(alpha = 0.6f))
                    }
                    IconButton({ q.move(i, -1) }, enabled = i > 0) { Icon(Icons.Rounded.KeyboardArrowUp, "Move up") }
                    IconButton({ q.move(i, 1) }, enabled = i < q.tracks.lastIndex) { Icon(Icons.Rounded.KeyboardArrowDown, "Move down") }
                    IconButton({ q.remove(i) }) { Icon(Icons.Rounded.Close, "Remove") }
                }
            }
        }
    }
}
