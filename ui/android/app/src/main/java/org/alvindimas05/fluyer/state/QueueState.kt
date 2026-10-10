package org.alvindimas05.fluyer.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import uniffi.fluyer_core.FluyerAppEngine
import uniffi.fluyer_core.TrackItemViewModel

/**
 * Play-queue snapshot plus queue commands. Row `index` is the queue position, not a
 * library index. Only re-read while [isOpen].
 */
class QueueState {
    var tracks by mutableStateOf(listOf<TrackItemViewModel>()); private set

    var engine: FluyerAppEngine? = null

    var isOpen by mutableStateOf(false)
        private set

    fun openSheet(open: Boolean) {
        if (open == isOpen) return
        isOpen = open
        if (open) reload()
    }

    fun reload() {
        val e = engine ?: return
        if (isOpen) tracks = e.queueGet()
    }

    fun goto(index: Int) {
        if (index !in tracks.indices) return
        engine?.queueGoto(index.toULong())
    }

    fun remove(index: Int) {
        if (index !in tracks.indices) return
        engine?.queueRemove(index.toULong())
        reload()
    }

    /** Move by [delta] rows; no-op past either end. */
    fun move(index: Int, delta: Int) = moveTo(index, index + delta)

    private fun moveTo(index: Int, target: Int) {
        if (index == target || index !in tracks.indices || target !in tracks.indices) return
        engine?.queueMove(index.toULong(), target.toULong())
        reload()
    }

    /** Stop playback and empty the queue. */
    fun clear() {
        if (tracks.isEmpty()) return
        engine?.queueClear()
        reload()
    }

    companion object {
        /**
         * `dropOffset` is the gap *before* removal, 0..count; dragging down lands one row
         * earlier once the source is taken out. (Port of the SwiftUI onMove helper.)
         */
        fun target(from: Int, dropOffset: Int) = if (dropOffset > from) dropOffset - 1 else dropOffset
    }
}
