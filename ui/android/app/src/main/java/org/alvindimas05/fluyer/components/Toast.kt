package org.alvindimas05.fluyer.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import org.alvindimas05.fluyer.state.AppState

/** Transient message from the core, shown above the player bar. */
@Composable
internal fun ToastPill(state: AppState, modifier: Modifier = Modifier) {
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
