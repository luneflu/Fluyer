package org.alvindimas05.fluyer.screens.home.albums

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.PlaylistAdd
import androidx.compose.material.icons.rounded.Shuffle
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.alvindimas05.fluyer.state.AppState

/** Header of the open album: back, name, and play / queue / shuffle. */
@Composable
internal fun AlbumHeader(state: AppState) {
    val sel = state.selection
    val d = sel.detail ?: return
    Row(Modifier.fillMaxWidth().padding(horizontal = 4.dp), verticalAlignment = Alignment.CenterVertically) {
        IconButton({ sel.clear() }) { Icon(Icons.AutoMirrored.Rounded.ArrowBack, "Back to library") }
        Column(Modifier.weight(1f)) {
            Text(d.header.name, maxLines = 1, overflow = TextOverflow.Ellipsis, style = MaterialTheme.typography.titleMedium)
            Text(d.subtitle.ifEmpty { d.header.artist }, maxLines = 1, overflow = TextOverflow.Ellipsis, fontSize = 12.sp, color = Color.White.copy(alpha = 0.6f))
        }
        IconButton({ sel.playSelected() }) { Icon(Icons.Rounded.PlayArrow, "Play album") }
        IconButton({ sel.queueSelected() }) { Icon(Icons.Rounded.PlaylistAdd, "Add album to queue") }
        IconButton({ sel.shuffleSelected() }) { Icon(Icons.Rounded.Shuffle, "Shuffle album") }
    }
}
