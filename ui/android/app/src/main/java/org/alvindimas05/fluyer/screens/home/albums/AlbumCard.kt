package org.alvindimas05.fluyer.screens.home.albums

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.alvindimas05.fluyer.components.Cover
import org.alvindimas05.fluyer.state.AppState
import org.alvindimas05.fluyer.support.ThumbnailKey
import uniffi.fluyer_core.AlbumCardViewModel

/** One album cover: tap opens it, long press opens the play / queue / shuffle menu. */
@OptIn(ExperimentalFoundationApi::class)
@Composable
internal fun AlbumCard(state: AppState, album: AlbumCardViewModel, modifier: Modifier) {
    val sel = state.selection
    var menu by remember { mutableStateOf(false) }
    val i = album.index.toInt()
    val selected = sel.index == i
    Column(modifier.clip(RoundedCornerShape(10.dp)).combinedClickable(onLongClick = { menu = true }) {
        if (selected) sel.clear() else sel.select(i)
    }.then(if (selected) Modifier.background(Color.White.copy(alpha = 0.14f)) else Modifier).padding(6.dp)) {
        Cover(state, ThumbnailKey.album(album.index, 300), 300, Modifier.fillMaxWidth().aspectRatio(1f)) {
            state.engine?.artworkLoadAlbumThumbnail(album.index, it)
        }
        Text(album.name, Modifier.padding(top = 6.dp), maxLines = 1, overflow = TextOverflow.Ellipsis, fontSize = 14.sp)
        Text(album.artist, maxLines = 1, overflow = TextOverflow.Ellipsis, fontSize = 12.sp, color = Color.White.copy(alpha = 0.6f))
        DropdownMenu(menu, { menu = false }) {
            DropdownMenuItem({ Text("Play") }, { sel.playAlbum(i); menu = false })
            DropdownMenuItem({ Text("Add to Queue") }, { sel.queueAlbum(i); menu = false })
            DropdownMenuItem({ Text("Shuffle") }, { sel.shuffleAlbum(i); menu = false })
        }
    }
}
