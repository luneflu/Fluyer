package org.alvindimas05.fluyer.screens.home.toolbar

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.QueueMusic
import androidx.compose.material.icons.rounded.Album
import androidx.compose.material.icons.rounded.MusicNote
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.alvindimas05.fluyer.state.AppState
import org.alvindimas05.fluyer.state.LibraryMode

/** Library toolbar: songs/albums toggle, search, sort, queue and settings. */
@Composable
internal fun TopBar(state: AppState, openSettings: () -> Unit) {
    val sel = state.selection
    var searching by remember { mutableStateOf(false) }
    Column(Modifier.fillMaxWidth().padding(horizontal = 8.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            SingleChoiceSegmentedButtonRow(Modifier.weight(1f).padding(end = 4.dp)) {
                SegmentedButton(
                    sel.mode == LibraryMode.TRACKS, { sel.mode = LibraryMode.TRACKS },
                    SegmentedButtonDefaults.itemShape(0, 2),
                    colors = segColors,
                    icon = {}, // no selected checkmark
                ) { Icon(Icons.Rounded.MusicNote, "Songs") }
                SegmentedButton(
                    sel.mode == LibraryMode.ALBUMS, { sel.mode = LibraryMode.ALBUMS },
                    SegmentedButtonDefaults.itemShape(1, 2),
                    colors = segColors,
                    icon = {},
                ) { Icon(Icons.Rounded.Album, "Albums") }
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
