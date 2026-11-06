package org.alvindimas05.fluyer.support

import uniffi.fluyer_core.ColorRgb
import uniffi.fluyer_core.NativeRepeatMode
import uniffi.fluyer_core.PlayViewModel
import uniffi.fluyer_core.PlayerBarViewModel
import uniffi.fluyer_core.ScanStatusViewModel

// Placeholders shown before the core reports anything; mirror the Rust constructors.
val NO_TRACK_BAR = PlayerBarViewModel(
    trackIndex = -1, title = "No Track", artist = "", album = "", positionMs = 0uL, durationMs = 0uL,
    progressPct = 0f, timeLabel = "0:00 / 0:00", isPlaying = false, repeatMode = NativeRepeatMode.NONE,
    isShuffled = false, volume = 1f,
)
val EMPTY_PLAY_VIEW = PlayViewModel(
    track = null, lyrics = emptyList(), currentLyricIndex = -1,
    palette = listOf(ColorRgb(28u.toUByte(), 28u.toUByte(), 36u.toUByte())),
)
val IDLE_SCAN = ScanStatusViewModel(
    isScanning = false, current = 0uL, total = 0uL, progressPct = 0f, statusLabel = "Ready",
)
