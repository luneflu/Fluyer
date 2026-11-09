package org.alvindimas05.fluyer.screens.home.albums

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import org.alvindimas05.fluyer.state.AppState

/** Albums mode: adaptive grid of album covers. */
@Composable
internal fun AlbumGrid(state: AppState) {
    val albums = state.selection.displayedAlbums
    LazyVerticalGrid(GridCells.Adaptive(150.dp), contentPadding = PaddingValues(12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        items(albums, key = { it.index.toLong() }) { AlbumCard(state, it, Modifier.fillMaxWidth()) }
    }
}
