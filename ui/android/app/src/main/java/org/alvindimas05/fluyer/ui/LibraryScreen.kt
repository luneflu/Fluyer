package org.alvindimas05.fluyer.ui

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.QueueMusic
import androidx.compose.material.icons.rounded.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.alvindimas05.fluyer.state.*
import uniffi.fluyer_core.AlbumCardViewModel
import uniffi.fluyer_core.TrackItemViewModel

@Composable
fun LibraryScreen(state: AppState, onAddFolder: () -> Unit, openSettings: () -> Unit) {
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
                else -> TrackPane(state)
            }
        }
        MiniPlayer(state)
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

@Composable
private fun TopBar(state: AppState, openSettings: () -> Unit) {
    val sel = state.selection
    var searching by remember { mutableStateOf(false) }
    Column(Modifier.fillMaxWidth().padding(horizontal = 8.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            SingleChoiceSegmentedButtonRow(Modifier.weight(1f).padding(end = 4.dp)) {
                SegmentedButton(
                    sel.mode == LibraryMode.TRACKS, { sel.mode = LibraryMode.TRACKS },
                    SegmentedButtonDefaults.itemShape(0, 2),
                    colors = segColors,
                ) { Text("Songs") }
                SegmentedButton(
                    sel.mode == LibraryMode.ALBUMS, { sel.mode = LibraryMode.ALBUMS },
                    SegmentedButtonDefaults.itemShape(1, 2),
                    colors = segColors,
                ) { Text("Albums") }
            }
            IconButton({ searching = !searching; if (!searching) sel.query = "" }) { Icon(Icons.Rounded.Search, "Search") }
            SortMenu(sel)
            IconButton({ state.queue.openSheet(true) }) { Icon(Icons.AutoMirrored.Rounded.QueueMusic, "Queue") }
            IconButton(openSettings) { Icon(Icons.Rounded.Settings, "Settings") }
        }
        if (searching) {
            BasicTextField(
                sel.query, { sel.query = it }, singleLine = true,
                textStyle = TextStyle(color = Color.White, fontSize = 16.sp),
                cursorBrush = SolidColor(Color.White),
                modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp).clip(RoundedCornerShape(10.dp))
                    .background(Color.White.copy(alpha = 0.12f)).padding(12.dp),
                decorationBox = { inner ->
                    if (sel.query.isEmpty()) Text("Search title, artist, album", color = Color.White.copy(alpha = 0.5f))
                    inner()
                },
            )
        }
    }
}

private val segColors
    @Composable get() = SegmentedButtonDefaults.colors(
        activeContainerColor = Color.White, activeContentColor = Color.Black,
        inactiveContainerColor = Color.Transparent, inactiveContentColor = Color.White,
    )

@Composable
private fun SortMenu(sel: AlbumSelection) {
    var open by remember { mutableStateOf(false) }
    IconButton({ open = true }) { Icon(Icons.Rounded.Sort, "Sort") }
    DropdownMenu(open, { open = false }) {
        if (sel.mode == LibraryMode.ALBUMS) {
            AlbumSort.entries.forEach {
                DropdownMenuItem({ Text(it.label) }, { sel.albumSort = it; open = false }, leadingIcon = { Check(sel.albumSort == it) })
            }
        } else {
            TrackSort.entries.forEach {
                DropdownMenuItem({ Text(it.label) }, { sel.trackSort = it; open = false }, leadingIcon = { Check(sel.trackSort == it) })
            }
        }
        HorizontalDivider()
        DropdownMenuItem({ Text("Ascending") }, { sel.sortAscending = true; open = false }, leadingIcon = { Check(sel.sortAscending) })
        DropdownMenuItem({ Text("Descending") }, { sel.sortAscending = false; open = false }, leadingIcon = { Check(!sel.sortAscending) })
    }
}

@Composable
private fun Check(on: Boolean) {
    if (on) Icon(Icons.Rounded.Check, null) else Spacer(Modifier.size(24.dp))
}

// Songs mode

@Composable
private fun TrackPane(state: AppState) {
    val sel = state.selection
    val tracks = sel.displayedTracks
    LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(bottom = 8.dp)) {
        item(key = "carousel") { AlbumCarousel(state) }
        if (sel.isActive) item(key = "header") { CollectionHeader(state) }
        items(tracks, key = { "${sel.index}-${it.index}-${it.path}" }) { TrackRow(state, it) }
    }
}

@Composable
private fun AlbumCarousel(state: AppState) {
    val sel = state.selection
    val albums = sel.displayedAlbums
    if (albums.isEmpty()) return
    LazyRow(contentPadding = PaddingValues(horizontal = 12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.padding(vertical = 8.dp)) {
        items(albums, key = { it.index.toLong() }) { AlbumCard(state, it, Modifier.width(140.dp)) }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun AlbumCard(state: AppState, album: AlbumCardViewModel, modifier: Modifier) {
    val sel = state.selection
    var menu by remember { mutableStateOf(false) }
    val i = album.index.toInt()
    val selected = sel.index == i
    Column(modifier.clip(RoundedCornerShape(10.dp)).combinedClickable(onLongClick = { menu = true }) {
        if (selected) sel.clear() else sel.select(i)
    }.then(if (selected) Modifier.background(Color.White.copy(alpha = 0.14f)) else Modifier).padding(6.dp)) {
        Cover(state, ThumbnailKey.album(album.index, 300), 300, Modifier.fillMaxWidth().aspectRatio(1f)) {
            state.engine?.loadAlbumThumbnail(album.index, it)
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

@Composable
private fun CollectionHeader(state: AppState) {
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

@Composable
private fun TrackRow(state: AppState, track: TrackItemViewModel) {
    val sel = state.selection
    val albumIndex = sel.index
    // Inside an album every row shows the album cover (one cache entry); else its own cover.
    val key = if (albumIndex != null) ThumbnailKey.album(albumIndex.toULong(), 88) else ThumbnailKey.track(track.index, 88)
    Row(
        Modifier.fillMaxWidth().combinedClickableTap { sel.playTrack(track) }.padding(horizontal = 16.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Cover(state, key, 88, Modifier.size(44.dp)) {
            if (albumIndex != null) state.engine?.loadAlbumThumbnail(albumIndex.toULong(), it)
            else state.engine?.loadTrackThumbnail(track.index, it)
        }
        Column(Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                if (track.isCurrent) {
                    Icon(
                        if (state.playback.bar.isPlaying) Icons.Rounded.VolumeUp else Icons.Rounded.VolumeMute,
                        "Now playing", Modifier.size(14.dp).padding(end = 4.dp),
                    )
                }
                Text(track.title, maxLines = 1, overflow = TextOverflow.Ellipsis, fontWeight = if (track.isCurrent) androidx.compose.ui.text.font.FontWeight.Bold else null)
            }
            Text(track.artist, maxLines = 1, overflow = TextOverflow.Ellipsis, fontSize = 12.sp, color = Color.White.copy(alpha = 0.6f))
        }
        Text(track.durationFormatted, fontSize = 12.sp, color = Color.White.copy(alpha = 0.6f))
    }
}

private fun Modifier.combinedClickableTap(onClick: () -> Unit) = this.then(Modifier.clickable(onClick = onClick))

// Albums mode

@Composable
private fun AlbumGrid(state: AppState) {
    val albums = state.selection.displayedAlbums
    LazyVerticalGrid(GridCells.Adaptive(150.dp), contentPadding = PaddingValues(12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        items(albums, key = { it.index.toLong() }) { AlbumCard(state, it, Modifier.fillMaxWidth()) }
    }
}
