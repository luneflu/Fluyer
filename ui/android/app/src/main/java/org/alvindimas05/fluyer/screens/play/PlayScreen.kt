package org.alvindimas05.fluyer.screens.play

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.KeyboardArrowDown
import androidx.compose.material.icons.rounded.Lyrics
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.unit.dp
import org.alvindimas05.fluyer.screens.play.controls.PlayControls
import org.alvindimas05.fluyer.screens.play.lyrics.Lyrics
import org.alvindimas05.fluyer.state.AppState

/** Full-screen now-playing: cover, seek, transport, volume, synced lyrics. */
@Composable
internal fun PlayScreen(state: AppState) {
    val landscape = LocalConfiguration.current.orientation == Configuration.ORIENTATION_LANDSCAPE
    var showLyrics by remember { mutableStateOf(false) }
    Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.6f)).safeDrawingPadding()) {
        IconButton({ state.playback.openPlayView(false) }) { Icon(Icons.Rounded.KeyboardArrowDown, "Close") }
        if (landscape) {
            Row(Modifier.fillMaxSize().padding(top = 40.dp, start = 16.dp, end = 16.dp), horizontalArrangement = Arrangement.spacedBy(24.dp)) {
                Column(Modifier.weight(1f).fillMaxHeight().verticalScroll(rememberScrollState())) { PlayControls(state, coverFraction = false) }
                Lyrics(state, Modifier.weight(1f).fillMaxHeight())
            }
        } else {
            Column(Modifier.fillMaxSize().padding(top = 40.dp, start = 24.dp, end = 24.dp, bottom = 12.dp)) {
                if (showLyrics) Lyrics(state, Modifier.weight(1f).fillMaxWidth())
                else Column(Modifier.weight(1f).fillMaxWidth(), verticalArrangement = Arrangement.Center) { PlayControls(state, coverFraction = true) }
                TextButton({ showLyrics = !showLyrics }, Modifier.align(Alignment.CenterHorizontally)) {
                    Icon(Icons.Rounded.Lyrics, null, Modifier.padding(end = 6.dp))
                    Text(if (showLyrics) "Controls" else "Lyrics")
                }
            }
        }
    }
}
