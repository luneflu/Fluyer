package org.alvindimas05.fluyer.backdrop

import android.app.ActivityManager
import android.content.Context
import android.graphics.Bitmap
import android.opengl.GLSurfaceView
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner

private val Grey = Color(0.2f, 0.2f, 0.21f)

/**
 * Animated, blurred backdrop of [artwork] (GLES 3 port of ui/macos ArtworkBackdropRenderer).
 * Null shows the music-note placeholder. Pauses with the lifecycle. The caller keeps ownership of [artwork].
 */
@Composable
fun AnimatedBackdrop(artwork: Bitmap?, reduceMotion: Boolean, modifier: Modifier = Modifier) {
    val context = LocalContext.current
    var failed by remember { mutableStateOf(false) }
    val supported = remember {
        (context.getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager)
            .deviceConfigurationInfo.reqGlEsVersion >= 0x30000
    }
    if (!supported || failed) {
        // ponytail: Modifier.blur is a no-op below API 31; plain scaled image there.
        Box(modifier.background(Grey)) {
            if (artwork != null) {
                Image(artwork.asImageBitmap(), null, Modifier.fillMaxSize().blur(48.dp), contentScale = ContentScale.Crop)
            }
        }
        return
    }

    val renderer = remember { BackdropRenderer { failed = true } }
    renderer.reduceMotion = reduceMotion
    LaunchedEffect(artwork) { renderer.setArtwork(artwork) }
    val view = remember {
        GLSurfaceView(context).apply {
            setEGLContextClientVersion(3)
            setEGLConfigChooser(8, 8, 8, 0, 0, 0)
            setRenderer(renderer)
            renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY // renderer sleeps to ~30 fps
        }
    }
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    DisposableEffect(lifecycle, view) {
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_RESUME) view.onResume()
            else if (event == Lifecycle.Event.ON_PAUSE) view.onPause()
        }
        lifecycle.addObserver(observer)
        onDispose {
            lifecycle.removeObserver(observer)
            view.onPause() // detaching ends the GL thread and frees the context with all GL objects
        }
    }
    Box(modifier.background(Grey)) { AndroidView({ view }, Modifier.fillMaxSize()) }
}
