package org.alvindimas05.fluyer.ui

import android.graphics.Bitmap
import android.os.Build
import android.provider.Settings
import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.MusicNote
import androidx.compose.material3.*
import androidx.compose.runtime.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import org.alvindimas05.fluyer.backdrop.AnimatedBackdrop
import org.alvindimas05.fluyer.state.AppState
import org.alvindimas05.fluyer.state.BackdropSource

private val Scheme = darkColorScheme(
    primary = Color.White, onPrimary = Color.Black,
    secondary = Color.White, onSecondary = Color.Black,
    surface = Color(0xFF1C1C1E), onSurface = Color.White,
    background = Color.Black, onBackground = Color.White,
    surfaceContainerHigh = Color(0xFF2A2A2D),
)

/** Root: backdrop behind everything, then library, with play / settings screens, queue sheet and toast over it. */
@Composable
fun FluyerApp(state: AppState, onAddFolder: () -> Unit, onRescan: () -> Unit) {
    MaterialTheme(colorScheme = Scheme) {
        Surface(color = Color.Transparent, contentColor = Color.White, modifier = Modifier.fillMaxSize()) {
            var showSettings by remember { mutableStateOf(false) }
            Box(Modifier.fillMaxSize()) {
                Backdrop(state)
                Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.35f)))

                // ponytail: library is dropped (not just covered) while a full screen is up, so it cannot show through the scrim.
                if (!state.playback.showPlayView && !showSettings) LibraryScreen(state, onAddFolder, openSettings = { showSettings = true })

                AnimatedVisibility(
                    state.playback.showPlayView,
                    enter = slideInVertically { it } + fadeIn(), exit = slideOutVertically { it } + fadeOut(),
                ) {
                    BackHandler { state.playback.openPlayView(false) }
                    PlayScreen(state)
                }

                AnimatedVisibility(showSettings, enter = fadeIn(), exit = fadeOut()) {
                    BackHandler { showSettings = false }
                    SettingsScreen(state, onAddFolder, onRescan, onBack = { showSettings = false })
                }

                QueueSheet(state)
                ToastPill(state, Modifier.align(Alignment.BottomCenter))
            }
        }
    }
}

/** Whole-window backdrop. Bitmap choice mirrors macOS AnimatedBackgroundView. */
@Composable
private fun Backdrop(state: AppState) {
    val settings = state.settings
    val playView = state.playback.playView
    val key = "${settings.backdropSource}|${playView.track?.path ?: "none"}|" +
        playView.palette.joinToString(";") { "${it.r},${it.g},${it.b}" }
    val artwork by produceState<Bitmap?>(null, key) {
        val engine = state.engine ?: return@produceState run { value = null }
        value = withContext(Dispatchers.Default) { backdropBitmap(state, engine, settings.backdropSource, playView.track != null) }
    }
    val ctx = LocalContext.current
    val reduceMotion = remember {
        Settings.Global.getFloat(ctx.contentResolver, Settings.Global.ANIMATOR_DURATION_SCALE, 1f) == 0f
    }
    BackdropContent(artwork, reduceMotion, settings.animatedBackground)
}

private suspend fun backdropBitmap(state: AppState, engine: uniffi.fluyer_core.FluyerAppEngine, source: BackdropSource, hasTrack: Boolean): Bitmap? {
        var bmp: Bitmap? = null
        if (source == BackdropSource.ARTWORK && hasTrack) {
            // ponytail: 1200px is ample for a full-screen blurred backdrop.
            bmp = engine.artworkLoadCurrentThumbnail(1200u)?.let { state.thumbnails.decode(it) }
        }
        // Blocks mode, no track, or coverless track: Rust gives palette/grey blocks.
        if (bmp == null) {
            val f = engine.backdropLoadBlockArtwork()
            val w = f.width.toInt()
            val h = f.height.toInt()
            if (w > 0 && h > 0 && f.rgba.size == w * h * 4) bmp = state.thumbnails.rgbaToBitmap(f.rgba, w, h)
        }
        return bmp
}

@Composable
private fun BackdropContent(artwork: Bitmap?, reduceMotion: Boolean, animated: Boolean) {
    if (animated) {
        AnimatedBackdrop(artwork, reduceMotion, Modifier.fillMaxSize())
    } else {
        // Static: blurred image where the platform can (API 31+), otherwise just dimmed.
        Box(Modifier.fillMaxSize().background(Color.Black)) {
            artwork?.let {
                val blur = if (Build.VERSION.SDK_INT >= 31) Modifier.blur(60.dp) else Modifier
                Image(it.asImageBitmap(), null, Modifier.fillMaxSize().then(blur), contentScale = ContentScale.Crop)
            }
        }
    }
}

@Composable
private fun ToastPill(state: AppState, modifier: Modifier) {
    val message = state.toast.message
    // Keep the last text while fading out.
    var shown by remember { mutableStateOf("") }
    if (message != null) shown = message
    AnimatedVisibility(message != null, modifier, enter = fadeIn(), exit = fadeOut()) {
        Surface(
            shape = RoundedCornerShape(50), color = Color(0xE6303034), contentColor = Color.White,
            modifier = Modifier.navigationBarsPadding().padding(bottom = 96.dp, start = 24.dp, end = 24.dp),
        ) {
            Text(shown, Modifier.padding(horizontal = 18.dp, vertical = 10.dp), maxLines = 2, overflow = TextOverflow.Ellipsis)
        }
    }
}

/** Cover thumbnail from the shared store, music-note placeholder until (or unless) one loads. */
@Composable
fun Cover(
    state: AppState, key: String, px: Int, modifier: Modifier = Modifier,
    load: suspend (UInt) -> ByteArray?,
) {
    val generation = state.thumbnails.generation
    val bmp by produceState(state.thumbnails.peek(key), key, generation) {
        value = state.thumbnails.image(key) { load(px.toUInt()) }
    }
    Box(modifier.clip(RoundedCornerShape(6.dp)).background(Color.White.copy(alpha = 0.08f)), contentAlignment = Alignment.Center) {
        val b = bmp
        if (b != null) Image(b.asImageBitmap(), null, Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
        else Icon(Icons.Rounded.MusicNote, null, tint = Color.White.copy(alpha = 0.3f))
    }
}

/** Cover of the playing track. */
@Composable
fun CurrentCover(state: AppState, px: Int, modifier: Modifier = Modifier) {
    val path = state.playback.playView.track?.path ?: "idx${state.playback.bar.trackIndex}"
    Cover(state, org.alvindimas05.fluyer.state.ThumbnailKey.current(px, path), px, modifier) { state.engine?.artworkLoadCurrentThumbnail(it) }
}

@Composable
fun SettingsScreen(state: AppState, onAddFolder: () -> Unit, onRescan: () -> Unit, onBack: () -> Unit) {
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
