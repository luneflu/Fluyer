package org.alvindimas05.fluyer.screens.home.musicgrid

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import org.alvindimas05.fluyer.screens.home.albums.AlbumCarousel
import org.alvindimas05.fluyer.screens.home.albums.AlbumHeader
import org.alvindimas05.fluyer.state.AppState

/** Songs mode: album carousel, album header (when one is open) and the track rows. */
@Composable
internal fun TrackList(state: AppState) {
    val sel = state.selection
    val tracks = sel.displayedTracks
    LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(bottom = 8.dp)) {
        item(key = "carousel") { AlbumCarousel(state) }
        if (sel.isActive) item(key = "header") { AlbumHeader(state) }
        items(tracks, key = { "${sel.index}-${it.index}-${it.path}" }) { TrackRow(state, it) }
    }
}
