package org.alvindimas05.fluyer.screens.home.albums

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import org.alvindimas05.fluyer.state.AppState

/** Horizontal strip of album covers above the track list. */
@Composable
internal fun AlbumCarousel(state: AppState) {
    val sel = state.selection
    val albums = sel.displayedAlbums
    if (albums.isEmpty()) return
    LazyRow(contentPadding = PaddingValues(horizontal = 12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.padding(vertical = 8.dp)) {
        items(albums, key = { it.index.toLong() }) { AlbumCard(state, it, Modifier.width(140.dp)) }
    }
}
