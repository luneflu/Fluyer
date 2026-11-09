package org.alvindimas05.fluyer.screens.home.queue

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import org.alvindimas05.fluyer.state.AppState

/** Queue list: tap = goto, per-row remove / move up / move down, header clear. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun QueueSheet(state: AppState) {
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
            itemsIndexed(q.tracks, key = { i, t -> "$i-${t.path}" }) { i, t -> QueueRow(state, i, t) }
        }
    }
}
