package org.alvindimas05.fluyer.screens.home

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.LibraryMusic
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.alvindimas05.fluyer.screens.home.albums.AlbumGrid
import org.alvindimas05.fluyer.screens.home.musicgrid.TrackList
import org.alvindimas05.fluyer.screens.home.playerbar.PlayerBar
import org.alvindimas05.fluyer.screens.home.toolbar.TopBar
import org.alvindimas05.fluyer.state.AppState
import org.alvindimas05.fluyer.state.LibraryMode

/** Library screen: toolbar, song/album list, player bar. */
@Composable
internal fun HomeScreen(state: AppState, onAddFolder: () -> Unit, openSettings: () -> Unit) {
    val sel = state.selection
    Column(Modifier.fillMaxSize().safeDrawingPadding()) {
        TopBar(state, openSettings)
        if (state.library.scanStatus.isScanning) {
            LinearProgressIndicator(
                progress = { state.library.scanStatus.progressPct.coerceIn(0f, 1f) },
                modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp), color = Color.White,
            )
            Text(state.library.scanStatus.statusLabel, Modifier.padding(horizontal = 16.dp, vertical = 2.dp), fontSize = 12.sp, color = Color.White.copy(alpha = 0.7f))
        }
        Box(Modifier.weight(1f).fillMaxWidth()) {
            when {
                state.library.tracks.isEmpty() && !state.library.scanStatus.isScanning -> EmptyLibrary(onAddFolder)
                sel.mode == LibraryMode.ALBUMS -> AlbumGrid(state)
                else -> TrackList(state)
            }
        }
        PlayerBar(state)
    }
}

@Composable
private fun EmptyLibrary(onAddFolder: () -> Unit) {
    Column(Modifier.fillMaxSize(), verticalArrangement = Arrangement.Center, horizontalAlignment = Alignment.CenterHorizontally) {
        Icon(Icons.Rounded.LibraryMusic, null, Modifier.size(64.dp), tint = Color.White.copy(alpha = 0.4f))
        Text("Your library is empty", Modifier.padding(top = 12.dp, bottom = 16.dp))
        Button(onAddFolder) { Text("Add music folder") }
    }
}
