package org.alvindimas05.fluyer

import android.app.Application
import android.content.Intent
import androidx.compose.runtime.snapshotFlow
import androidx.core.content.ContextCompat
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.launch
import org.alvindimas05.fluyer.state.AppState

/** Process-wide owner of the engine and [AppState], shared by the Activity and [PlaybackService]. */
class FluyerApplication : Application() {
    lateinit var state: AppState
        private set

    /** Saved folders are re-scanned once per process, not per Activity re-creation. */
    var scannedSavedFolders = false

    override fun onCreate() {
        super.onCreate()
        state = AppState(filesDir, cacheDir)
        // Start the media service whenever playback begins; it manages itself from there.
        state.scope.launch {
            snapshotFlow { state.playback.bar.isPlaying }.filter { it }.collect {
                runCatching {
                    ContextCompat.startForegroundService(this@FluyerApplication, Intent(this@FluyerApplication, PlaybackService::class.java))
                }.onFailure { android.util.Log.e("Fluyer", "PlaybackService start failed", it) }
            }
        }
    }
}
