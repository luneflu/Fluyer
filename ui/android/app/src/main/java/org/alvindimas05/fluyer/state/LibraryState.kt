package org.alvindimas05.fluyer.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.fluyer_core.AlbumCardViewModel
import uniffi.fluyer_core.FluyerAppEngine
import uniffi.fluyer_core.ScanStatusViewModel
import uniffi.fluyer_core.TrackItemViewModel

/** The scanned library: flat track list, album list and scan progress. */
class LibraryState {
    var tracks by mutableStateOf(listOf<TrackItemViewModel>())
    var albums by mutableStateOf(listOf<AlbumCardViewModel>())
    var scanStatus by mutableStateOf(IDLE_SCAN)

    var engine: FluyerAppEngine? = null

    /**
     * Re-read the whole library from the core. Runs the per-row FFI calls on Dispatchers.Default
     * (one call per track/album blocks the main thread for seconds on a big or scanning library),
     * then publishes on the caller's (main) dispatcher.
     */
    suspend fun reload() {
        val e = engine ?: return
        val (status, t, a) = withContext(Dispatchers.Default) {
            Triple(
                e.getScanStatus(),
                (0 until e.getTrackCount().toInt()).mapNotNull { e.getTrackView(it.toULong()) },
                (0 until e.getAlbumCount().toInt()).mapNotNull { e.getAlbumCard(it.toULong()) },
            )
        }
        scanStatus = status
        tracks = t
        albums = a
    }
}
