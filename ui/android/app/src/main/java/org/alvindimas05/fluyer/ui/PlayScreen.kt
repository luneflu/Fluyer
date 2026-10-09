package org.alvindimas05.fluyer.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.VolumeUp
import androidx.compose.material.icons.rounded.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import android.content.res.Configuration
import org.alvindimas05.fluyer.state.AppState
import org.alvindimas05.fluyer.state.PlaybackIcons
import org.alvindimas05.fluyer.state.TimeFormat
import uniffi.fluyer_core.NativeRepeatMode

/** Full-screen now-playing: cover, seek, transport, volume, synced lyrics. */
@Composable
fun PlayScreen(state: AppState) {
    val landscape = LocalConfiguration.current.orientation == Configuration.ORIENTATION_LANDSCAPE
    var showLyrics by remember { mutableStateOf(false) }
    Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.6f)).safeDrawingPadding()) {
        IconButton({ state.playback.openPlayView(false) }) { Icon(Icons.Rounded.KeyboardArrowDown, "Close") }
        if (landscape) {
            Row(Modifier.fillMaxSize().padding(top = 40.dp, start = 16.dp, end = 16.dp), horizontalArrangement = Arrangement.spacedBy(24.dp)) {
                Column(Modifier.weight(1f).fillMaxHeight().verticalScroll(rememberScrollState())) { Controls(state, coverFraction = false) }
                Lyrics(state, Modifier.weight(1f).fillMaxHeight())
            }
        } else {
            Column(Modifier.fillMaxSize().padding(top = 40.dp, start = 24.dp, end = 24.dp, bottom = 12.dp)) {
                if (showLyrics) Lyrics(state, Modifier.weight(1f).fillMaxWidth())
                else Column(Modifier.weight(1f).fillMaxWidth(), verticalArrangement = Arrangement.Center) { Controls(state, coverFraction = true) }
                TextButton({ showLyrics = !showLyrics }, Modifier.align(Alignment.CenterHorizontally)) {
                    Icon(Icons.Rounded.Lyrics, null, Modifier.padding(end = 6.dp))
                    Text(if (showLyrics) "Controls" else "Lyrics")
                }
            }
        }
    }
}

@Composable
private fun Controls(state: AppState, coverFraction: Boolean) {
    val pb = state.playback
    val bar = pb.bar
    val clock = pb.clock
    Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.fillMaxWidth()) {
        CurrentCover(state, 600, if (coverFraction) Modifier.fillMaxWidth(0.8f).aspectRatio(1f) else Modifier.height(96.dp).aspectRatio(1f))
        Spacer(Modifier.height(16.dp))
        Text(bar.title, fontSize = 20.sp, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis, textAlign = TextAlign.Center)
        Text(listOf(bar.artist, bar.album).filter { it.isNotEmpty() }.joinToString(" · "), maxLines = 1, overflow = TextOverflow.Ellipsis, color = Color.White.copy(alpha = 0.7f))

        // Local drag value so the thumb does not fight the 4 Hz clock while scrubbing.
        var dragging by remember { mutableStateOf<Float?>(null) }
        Slider(
            dragging ?: clock.progressPct.coerceIn(0f, 1f),
            { dragging = it },
            onValueChangeFinished = { dragging?.let { pb.seek(it) }; dragging = null },
            colors = sliderColors,
        )
        Row(Modifier.fillMaxWidth().padding(horizontal = 4.dp)) {
            val shownPos = dragging?.let { clock.targetPositionMs(it) } ?: clock.positionMs
            Text(TimeFormat.elapsed(shownPos), fontSize = 12.sp)
            Spacer(Modifier.weight(1f))
            Text(TimeFormat.elapsed(clock.durationMs), fontSize = 12.sp)
        }

        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceEvenly, verticalAlignment = Alignment.CenterVertically) {
            IconButton({ pb.shuffle() }) {
                Icon(Icons.Rounded.Shuffle, "Shuffle", tint = if (bar.isShuffled) Color.White else Color.White.copy(alpha = 0.45f))
            }
            IconButton({ pb.previous() }) { Icon(Icons.Rounded.SkipPrevious, "Previous", Modifier.size(36.dp)) }
            FilledIconButton({ pb.togglePlay() }, Modifier.size(64.dp), colors = IconButtonDefaults.filledIconButtonColors(containerColor = Color.White, contentColor = Color.Black)) {
                Icon(if (bar.isPlaying) Icons.Rounded.Pause else Icons.Rounded.PlayArrow, if (bar.isPlaying) "Pause" else "Play", Modifier.size(36.dp))
            }
            IconButton({ pb.next() }) { Icon(Icons.Rounded.SkipNext, "Next", Modifier.size(36.dp)) }
            IconButton({ pb.cycleRepeat() }) {
                Icon(
                    if (PlaybackIcons.repeat(bar.repeatMode) == PlaybackIcons.Repeat.REPEAT_ONE) Icons.Rounded.RepeatOne else Icons.Rounded.Repeat,
                    "Repeat", tint = if (bar.repeatMode == NativeRepeatMode.NONE) Color.White.copy(alpha = 0.45f) else Color.White,
                )
            }
        }

        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            IconButton({ pb.toggleMute() }) {
                Icon(
                    when (PlaybackIcons.volume(bar.volume)) {
                        PlaybackIcons.Volume.MUTE -> Icons.Rounded.VolumeOff
                        PlaybackIcons.Volume.LOW -> Icons.Rounded.VolumeMute
                        PlaybackIcons.Volume.MID -> Icons.Rounded.VolumeDown
                        PlaybackIcons.Volume.HIGH -> Icons.Rounded.VolumeUp
                    },
                    "Mute",
                )
            }
            Slider(bar.volume, { pb.setVolume(it) }, colors = sliderColors, modifier = Modifier.weight(1f))
        }
    }
}

private val sliderColors
    @Composable get() = SliderDefaults.colors(thumbColor = Color.White, activeTrackColor = Color.White, inactiveTrackColor = Color.White.copy(alpha = 0.25f))

/** Synced lyrics; the active line follows the clock and a tap seeks to a line. */
@Composable
private fun Lyrics(state: AppState, modifier: Modifier) {
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
