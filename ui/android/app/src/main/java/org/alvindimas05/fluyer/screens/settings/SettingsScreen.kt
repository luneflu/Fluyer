package org.alvindimas05.fluyer.screens.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import org.alvindimas05.fluyer.state.AppState
import org.alvindimas05.fluyer.state.BackdropSource

/** Settings: music folders, rescan, animated background, backdrop source. */
@Composable
internal fun SettingsScreen(state: AppState, onAddFolder: () -> Unit, onRescan: () -> Unit, onBack: () -> Unit) {
    val s = state.settings
    Column(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.85f)).safeDrawingPadding()) {
        Row(Modifier.fillMaxWidth().padding(horizontal = 4.dp), verticalAlignment = Alignment.CenterVertically) {
            IconButton(onBack) { Icon(Icons.AutoMirrored.Rounded.ArrowBack, "Back") }
            Text("Settings", style = MaterialTheme.typography.titleLarge)
        }
        Column(Modifier.weight(1f).verticalScroll(rememberScrollState()).padding(horizontal = 16.dp)) {
            Text("Music folders", style = MaterialTheme.typography.titleMedium, modifier = Modifier.padding(vertical = 8.dp))
            if (s.musicFolders.isEmpty()) Text("No folders yet", color = Color.White.copy(alpha = 0.6f))
            s.musicFolders.forEach { folder ->
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    Text(folder, Modifier.weight(1f), maxLines = 2, overflow = TextOverflow.Ellipsis)
                    IconButton({ state.removeFolder(folder) }) { Icon(Icons.Rounded.Close, "Remove $folder") }
                }
            }
            Row(Modifier.padding(vertical = 8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(onAddFolder) { Text("Add folder") }
                OutlinedButton(onRescan, enabled = s.musicFolders.isNotEmpty()) { Text("Rescan") }
            }
            HorizontalDivider(Modifier.padding(vertical = 12.dp))
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("Animated background", Modifier.weight(1f))
                Switch(s.animatedBackground, { s.animatedBackground = it })
            }
            Text("Backdrop source", style = MaterialTheme.typography.titleMedium, modifier = Modifier.padding(top = 16.dp, bottom = 8.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                BackdropSource.entries.forEach {
                    FilterChip(s.backdropSource == it, { s.backdropSource = it }, { Text(it.label) })
                }
            }
        }
    }
}
