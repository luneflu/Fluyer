package org.alvindimas05.fluyer.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import uniffi.fluyer_core.FluyerAppEngine
import uniffi.fluyer_core.PlayerBarViewModel

/**
 * Locally-ticked playback position and the active lyric cursor.
 *
 * ponytail: the core only emits PlayerBarUpdated on discrete transport actions, never on
 * a timer, so continuous progress is sampled UI-side. Kept apart from [PlaybackState.bar]
 * because it changes ~4x a second: a tick invalidates only widgets that read position.
 */
class PlaybackClock(private val scope: CoroutineScope) {
    var positionMs by mutableStateOf(0uL); private set
    var durationMs by mutableStateOf(0uL); private set
    var progressPct by mutableFloatStateOf(0f); private set
    var timeLabel by mutableStateOf(TimeFormat.pair(0uL, 0uL)); private set

    /**
     * Index into PlayViewModel.lyrics. Owned here, not read off the core's play_view
     * snapshot, because it advances at tick rate.
     */
    var currentLyricIndex by mutableIntStateOf(-1)

    /** Whether to advance [currentLyricIndex]. Only the now-playing screen needs it. */
    var followsLyrics = false

    private var engine: FluyerAppEngine? = null
    private var job: Job? = null

    fun start(sampling: FluyerAppEngine?) {
        engine = sampling
        if (job != null || sampling == null) return
        job = scope.launch {
            while (isActive) {
                tick()
                delay(TICK_MS)
            }
        }
    }

    fun stop() {
        job?.cancel()
        job = null
    }

    /** Adopt the position fields of an authoritative core snapshot verbatim. */
    fun adopt(snapshot: PlayerBarViewModel) {
        positionMs = snapshot.positionMs
        durationMs = snapshot.durationMs
        progressPct = snapshot.progressPct
        timeLabel = snapshot.timeLabel
    }

    /** Recompute derived position fields after a local seek, before the core echoes anything back. */
    fun applyLocalPosition(positionMs: ULong) {
        this.positionMs = positionMs
        if (durationMs == 0uL) return
        progressPct = positionMs.toFloat() / durationMs.toFloat()
        timeLabel = TimeFormat.pair(positionMs, durationMs)
    }

    /** Position targeted by a drag on the progress bar, or null when there is no duration. */
    fun targetPositionMs(fraction: Float): ULong? {
        if (durationMs == 0uL) return null
        return (durationMs.toFloat() * fraction.clamped(0f, 1f)).toULong()
    }

    private fun tick() {
        val e = engine ?: return
        val pos = e.getPosition()
        positionMs = pos
        if (durationMs == 0uL) return
        progressPct = pos.toFloat() / durationMs.toFloat()
        timeLabel = TimeFormat.pair(pos, durationMs)
        if (followsLyrics) currentLyricIndex = e.getActiveLyricIndex(pos)
    }

    private companion object {
        const val TICK_MS = 250L
    }
}
