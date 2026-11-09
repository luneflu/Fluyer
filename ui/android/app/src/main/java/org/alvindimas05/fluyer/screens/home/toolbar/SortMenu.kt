package org.alvindimas05.fluyer.screens.home.toolbar

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.Sort
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import org.alvindimas05.fluyer.state.AlbumSelection
import org.alvindimas05.fluyer.state.AlbumSort
import org.alvindimas05.fluyer.state.LibraryMode
import org.alvindimas05.fluyer.state.TrackSort

/** Sort menu: the keys of the active mode, then ascending / descending. */
@Composable
internal fun SortMenu(sel: AlbumSelection) {
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
