package org.alvindimas05.fluyer.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.MusicNote
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import org.alvindimas05.fluyer.state.AppState
import org.alvindimas05.fluyer.support.ThumbnailKey

/** Cover thumbnail from the shared store, music-note placeholder until (or unless) one loads. */
@Composable
internal fun Cover(
    state: AppState, key: String, px: Int, modifier: Modifier = Modifier,
    load: suspend (UInt) -> ByteArray?,
) {
    val generation = state.thumbnails.generation
    val bmp by produceState(state.thumbnails.peek(key), key, generation) {
        value = state.thumbnails.image(key) { load(px.toUInt()) }
    }
    Box(modifier.clip(RoundedCornerShape(6.dp)).background(Color.White.copy(alpha = 0.08f)), contentAlignment = Alignment.Center) {
        val b = bmp
        if (b != null) Image(b.asImageBitmap(), null, Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
        else Icon(Icons.Rounded.MusicNote, null, tint = Color.White.copy(alpha = 0.3f))
    }
}

/** Cover of the playing track. */
@Composable
internal fun CurrentCover(state: AppState, px: Int, modifier: Modifier = Modifier) {
    val path = state.playback.playView.track?.path ?: "idx${state.playback.bar.trackIndex}"
    Cover(state, ThumbnailKey.current(px, path), px, modifier) { state.engine?.artworkLoadCurrentThumbnail(it) }
}
