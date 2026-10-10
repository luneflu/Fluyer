package org.alvindimas05.fluyer.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.coroutines.CoroutineScope
import uniffi.fluyer_core.FluyerAppEngine
import uniffi.fluyer_core.PlayViewModel
import uniffi.fluyer_core.PlayerBarViewModel

/**
 * Now-playing state: metadata, transport flags, the position clock and every command the
 * UI issues to the player. The engine is assigned once by [AppState].
 */
class PlaybackState(scope: CoroutineScope) {
    /** Metadata and transport flags. Position lives in [clock]. */
    var bar by mutableStateOf(NO_TRACK_BAR); private set

    /** Core's now-playing snapshot. Its currentLyricIndex is unused; see [PlaybackClock.currentLyricIndex]. */
    var playView by mutableStateOf(EMPTY_PLAY_VIEW); private set

    /** Whether the full-screen now-playing surface is presented. */
    var showPlayView by mutableStateOf(false)
        private set

    val clock = PlaybackClock(scope)

    /** Bumps on every seek so the system media session can republish the jumped position. */
    var seekEpoch by mutableStateOf(0)
        private set

    var engine: FluyerAppEngine? = null

    private var lastAudibleVolume = 1f

    fun openPlayView(open: Boolean) {
        if (open == showPlayView) return
        showPlayView = open
        clock.followsLyrics = open
        if (open) syncLyricCursor()
    }

    /**
     * Adopt an authoritative player-bar snapshot.
     *
     * ponytail: FluyerEvent.PlayerBarUpdated does NOT carry the full view model (empty
     * title/artist/album, volume hardcoded 1.0), so callers re-read getPlayerBarView().
     */
    fun applyBar(snapshot: PlayerBarViewModel) {
        bar = snapshot
        clock.adopt(snapshot)
        if (showPlayView) reloadPlayView()
        syncClockToTransport()
    }

    fun reloadBar() {
        engine?.let { applyBar(it.playerGetBar()) }
    }

    fun reloadPlayView() {
        engine?.let { playView = it.playerGetPlayView() }
    }

    fun applyPlayView(vm: PlayViewModel) { playView = vm }

    fun applyTrackChange() {
        if (engine == null) return
        reloadBar()
        reloadPlayView()
        syncLyricCursor()
    }

    // Transport

    fun togglePlay() {
        engine?.playerTogglePlay()
        // Optimistic: the core confirms with its own PlayerBarUpdated.
        bar = bar.copy(isPlaying = !bar.isPlaying)
        syncClockToTransport()
    }

    /** Idempotent play/pause for system media controls, which name the target state. */
    fun play() { if (!bar.isPlaying) togglePlay() }
    fun pause() { if (bar.isPlaying) togglePlay() }
    fun next() { engine?.playerNext() }
    fun previous() { engine?.playerPrevious() }
    fun cycleRepeat() { engine?.playerCycleRepeat() }
    fun shuffle() { engine?.playerShuffle() }

    fun seek(toMs: ULong) {
        val e = engine ?: return
        e.playerSeek(toMs)
        clock.applyLocalPosition(toMs)
        seekEpoch++
        syncLyricCursor()
    }

    /** Seek from a fraction of the track, as produced by dragging the progress bar. */
    fun seek(fraction: Float) {
        clock.targetPositionMs(fraction)?.let { seek(it) }
    }

    fun setVolume(volume: Float) {
        val v = volume.clamped(0f, 1f)
        if (v > AUDIBLE) lastAudibleVolume = v
        bar = bar.copy(volume = v)
        engine?.playerSetVolume(v)
    }

    fun toggleMute() {
        if (bar.volume > AUDIBLE) setVolume(0f) else setVolume(maxOf(lastAudibleVolume, MIN_RESTORE))
    }

    private fun syncClockToTransport() {
        if (bar.isPlaying) clock.start(engine) else clock.stop()
    }

    private fun syncLyricCursor() {
        val e = engine ?: return
        if (!showPlayView) return
        clock.currentLyricIndex = e.lyricsGetActiveIndex(clock.positionMs)
    }

    private companion object {
        const val AUDIBLE = 0.001f
        const val MIN_RESTORE = 0.05f
    }
}
