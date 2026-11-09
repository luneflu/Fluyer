package org.alvindimas05.fluyer.screens.play.controls

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Pause
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Repeat
import androidx.compose.material.icons.rounded.RepeatOne
import androidx.compose.material.icons.rounded.Shuffle
import androidx.compose.material.icons.rounded.SkipNext
import androidx.compose.material.icons.rounded.SkipPrevious
import androidx.compose.material.icons.rounded.VolumeDown
import androidx.compose.material.icons.rounded.VolumeMute
import androidx.compose.material.icons.rounded.VolumeOff
import androidx.compose.material.icons.rounded.VolumeUp
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.alvindimas05.fluyer.components.CurrentCover
import org.alvindimas05.fluyer.state.AppState
import org.alvindimas05.fluyer.support.PlaybackIcons
import org.alvindimas05.fluyer.support.TimeFormat
import uniffi.fluyer_core.NativeRepeatMode

/** Now-playing controls: cover, title, progress, transport, volume. */
@Composable
internal fun PlayControls(state: AppState, coverFraction: Boolean) {
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
