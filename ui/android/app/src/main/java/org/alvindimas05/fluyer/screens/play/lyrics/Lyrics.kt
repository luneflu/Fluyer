package org.alvindimas05.fluyer.screens.play.lyrics

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.alvindimas05.fluyer.state.AppState

/** Synced lyrics; the active line follows the clock and a tap seeks to a line. */
@Composable
internal fun Lyrics(state: AppState, modifier: Modifier) {
    val lyrics = state.playback.playView.lyrics
    val active = state.playback.clock.currentLyricIndex
    val list = rememberLazyListState()
    LaunchedEffect(active) {
        if (active in lyrics.indices) list.animateScrollToItem((active - 2).coerceAtLeast(0))
    }
    if (lyrics.isEmpty()) {
        Box(modifier, contentAlignment = Alignment.Center) { Text("No lyrics", color = Color.White.copy(alpha = 0.5f)) }
        return
    }
    LazyColumn(modifier, list, contentPadding = PaddingValues(vertical = 48.dp)) {
        itemsIndexed(lyrics) { i, line ->
            Text(
                line.text.ifEmpty { "…" },
                Modifier.fillMaxWidth().clickable { state.playback.seek(line.timestampMs) }.padding(vertical = 8.dp),
                fontSize = if (i == active) 22.sp else 18.sp,
                fontWeight = if (i == active) FontWeight.Bold else FontWeight.Normal,
                color = Color.White.copy(alpha = if (i == active) 1f else 0.5f),
                textAlign = TextAlign.Center,
            )
        }
    }
}
