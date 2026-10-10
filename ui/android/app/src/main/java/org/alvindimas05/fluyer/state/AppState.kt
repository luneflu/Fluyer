package org.alvindimas05.fluyer.state

import android.os.Handler
import android.os.Looper
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import uniffi.fluyer_core.FluyerAppEngine
import uniffi.fluyer_core.FluyerEvent
import uniffi.fluyer_core.FluyerEventListener
import java.io.File

/**
 * Root of the app's observable state. Ownership is split by concern:
 *
 * - [playback]  now-playing metadata, transport commands, position clock
 * - [library]   scanned tracks and albums, scan progress
 * - [selection] which album, if any, the list is showing
 * - [toast]     transient messages from the core
 * - [queue]     play-queue snapshot for the queue sheet
 * - [settings]  persisted user settings (folders, toggles, volume)
 *
 * Owns the engine wiring, translates [FluyerEvent]s into updates on those objects and
 * performs the cross-cutting full-library refresh. All mutation happens on the main thread.
 */
class AppState(dataDir: File, cacheDir: File, createEngine: Boolean = true) : FluyerEventListener {
    val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    val library = LibraryState()
    val playback = PlaybackState(scope)
    val selection = AlbumSelection(library)
    val toast = ToastState(scope)
    val queue = QueueState()
    val settings = SettingsState(File(dataDir, SettingsState.FILE_NAME))
    val thumbnails = ThumbnailStore()

    var engine: FluyerAppEngine? = null
        private set

    private val main = Handler(Looper.getMainLooper())

    init {
        // Every property is initialised, so `this` can be handed to the core as listener.
        if (createEngine) {
            attach(runCatching { FluyerAppEngine(dataDir.absolutePath, cacheDir.absolutePath, this) }
                .onFailure { android.util.Log.e("Fluyer", "core failed to start", it) }
                .getOrNull())
        }
    }

    /**
     * Bind the engine to every state object, then restore persisted settings and re-scan
     * saved folders so files changed while closed show up (unchanged files are skipped by mtime).
     * Scanning is deferred until [scanSavedFolders] is called by the UI once permission is granted.
     */
    fun attach(engine: FluyerAppEngine?) {
        this.engine = engine
        playback.engine = engine
        library.engine = engine
        selection.engine = engine
        queue.engine = engine

        refresh()
        if (engine == null) return
        playback.setVolume(settings.volume)
    }

    private var libraryJob: Job? = null

    /** Re-read everything the window shows from the core. */
    fun refresh() {
        playback.reloadBar()
        playback.reloadPlayView()
        reloadLibrary()
        selection.reload()
        queue.reload()
    }

    /** Coalesced: a newer request cancels the in-flight read so scan bursts do not pile up. */
    private fun reloadLibrary() {
        libraryJob?.cancel()
        libraryJob = scope.launch { library.reload() }
    }

    fun scanFolders(paths: List<String>) {
        if (paths.isEmpty()) return
        settings.addFolders(paths)
        engine?.libraryScan(paths.map(SettingsState::normalize))
    }

    /** Re-scan every saved folder. */
    fun scanSavedFolders() {
        if (settings.musicFolders.isEmpty()) return
        engine?.libraryScan(settings.musicFolders)
    }

    /** Forget a folder and drop its tracks from the library. */
    fun removeFolder(path: String) {
        val stored = settings.removeFolder(path) ?: return
        engine?.libraryRemoveFolder(stored)
        refresh()
    }

    /** Play the whole library from the first track. */
    fun playAll() {
        if (library.tracks.isEmpty()) return
        engine?.libraryPlayAll(0uL)
    }

    /** Persist session state that changes too often to save live (volume). */
    fun saveSession() {
        settings.volume = playback.bar.volume
    }

    /** The core calls this from its own threads; hop to main once and mutate there. */
    override fun onEvent(event: FluyerEvent) {
        main.post { handle(event) }
    }

    private fun handle(event: FluyerEvent) {
        when (event) {
            is FluyerEvent.PlayerBarUpdated -> {
                playback.reloadBar()
                queue.reload()
            }
            // ponytail: declared but never emitted by the core; handled so it works once it is.
            is FluyerEvent.PlayViewUpdated -> playback.applyPlayView(event.vm)
            is FluyerEvent.TrackChanged -> {
                playback.applyTrackChange()
                // ponytail: full coalesced reload instead of a per-row flag patch; both are N FFI calls, now off-main.
                reloadLibrary()
                queue.reload()
            }
            FluyerEvent.LibraryUpdated -> refresh()
            is FluyerEvent.ScanProgress -> library.scanStatus = event.vm
            is FluyerEvent.Toast -> toast.show(event.message)
            is FluyerEvent.TrackCoverLoaded -> {
                thumbnails.invalidate(ThumbnailKey.trackPrefix(event.index))
                thumbnails.invalidate(ThumbnailKey.CURRENT_PREFIX)
                playback.reloadPlayView()
            }
            is FluyerEvent.AlbumCoverLoaded -> thumbnails.invalidate(ThumbnailKey.albumPrefix(event.index))
            is FluyerEvent.LyricsLoaded -> playback.reloadPlayView()
        }
    }
}
