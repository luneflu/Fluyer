package org.alvindimas05.fluyer

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import org.alvindimas05.fluyer.components.Backdrop
import org.alvindimas05.fluyer.components.ToastPill
import org.alvindimas05.fluyer.screens.home.HomeScreen
import org.alvindimas05.fluyer.screens.home.queue.QueueSheet
import org.alvindimas05.fluyer.screens.play.PlayScreen
import org.alvindimas05.fluyer.screens.settings.SettingsScreen
import org.alvindimas05.fluyer.state.AppState

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
                if (!state.playback.showPlayView && !showSettings) HomeScreen(state, onAddFolder, openSettings = { showSettings = true })

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
