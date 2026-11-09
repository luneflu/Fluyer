package org.alvindimas05.fluyer.components

import android.graphics.Bitmap
import android.os.Build
import android.provider.Settings
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.alvindimas05.fluyer.components.backdrop.AnimatedBackdrop
import org.alvindimas05.fluyer.state.AppState
import org.alvindimas05.fluyer.state.BackdropSource

/** Whole-window backdrop. Bitmap choice mirrors macOS AnimatedBackgroundView. */
@Composable
internal fun Backdrop(state: AppState) {
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
