package org.alvindimas05.fluyer.screens.home.musicgrid

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.VolumeMute
import androidx.compose.material.icons.rounded.VolumeUp
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.alvindimas05.fluyer.components.Cover
import org.alvindimas05.fluyer.state.AppState
import org.alvindimas05.fluyer.support.ThumbnailKey
import uniffi.fluyer_core.TrackItemViewModel

/** One track row: cover, title, artist, duration. */
@Composable
internal fun TrackRow(state: AppState, track: TrackItemViewModel) {
    val sel = state.selection
    val albumIndex = sel.index
    // Inside an album every row shows the album cover (one cache entry); else its own cover.
    val key = if (albumIndex != null) ThumbnailKey.album(albumIndex.toULong(), 88) else ThumbnailKey.track(track.index, 88)
    Row(
        Modifier.fillMaxWidth().clickable { sel.playTrack(track) }.padding(horizontal = 16.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Cover(state, key, 88, Modifier.size(44.dp)) {
            if (albumIndex != null) state.engine?.artworkLoadAlbumThumbnail(albumIndex.toULong(), it)
            else state.engine?.artworkLoadTrackThumbnail(track.index, it)
        }
        Column(Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                if (track.isCurrent) {
                    Icon(
                        if (state.playback.bar.isPlaying) Icons.Rounded.VolumeUp else Icons.Rounded.VolumeMute,
                        "Now playing", Modifier.size(14.dp).padding(end = 4.dp),
                    )
                }
                Text(track.title, maxLines = 1, overflow = TextOverflow.Ellipsis, fontWeight = if (track.isCurrent) FontWeight.Bold else null)
            }
            Text(track.artist, maxLines = 1, overflow = TextOverflow.Ellipsis, fontSize = 12.sp, color = Color.White.copy(alpha = 0.6f))
        }
        Text(track.durationFormatted, fontSize = 12.sp, color = Color.White.copy(alpha = 0.6f))
    }
}
