package org.alvindimas05.fluyer

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.ServiceInfo
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.media.AudioAttributes
import android.media.AudioManager
import android.os.Build
import android.support.v4.media.MediaMetadataCompat
import android.support.v4.media.session.MediaSessionCompat
import android.support.v4.media.session.PlaybackStateCompat
import androidx.compose.runtime.snapshotFlow
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import androidx.media.AudioAttributesCompat
import androidx.media.AudioFocusRequestCompat
import androidx.media.AudioManagerCompat
import androidx.media.app.NotificationCompat.MediaStyle
import androidx.media.session.MediaButtonReceiver
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import android.app.Service
import android.os.IBinder
import org.alvindimas05.fluyer.state.AppState

/**
 * Foreground media service: MediaSession + MediaStyle notification + audio focus.
 * Port of macOS NowPlayingCoordinator. Playback itself runs in the core; this only
 * mirrors [AppState.playback] to the system and routes system commands back.
 */
class PlaybackService : Service() {
    private val state: AppState get() = (application as FluyerApplication).state
    private lateinit var session: MediaSessionCompat
    private lateinit var audio: AudioManager
    private var observer: Job? = null
    private var coverJob: Job? = null
    private var cover: Bitmap? = null
    private var trackKey = ""
    private var foreground = false
    private var focus: AudioFocusRequestCompat? = null
    private var pausedByFocusLoss = false

    private val noisy = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            if (intent.action == AudioManager.ACTION_AUDIO_BECOMING_NOISY) state.playback.pause()
        }
    }

    override fun onCreate() {
        super.onCreate()
        audio = getSystemService(AUDIO_SERVICE) as AudioManager
        getSystemService(NotificationManager::class.java)
            .createNotificationChannel(NotificationChannel(CHANNEL, "Playback", NotificationManager.IMPORTANCE_LOW))
        session = MediaSessionCompat(this, "Fluyer").apply {
            setCallback(object : MediaSessionCompat.Callback() {
                override fun onPlay() = state.playback.play()
                override fun onPause() = state.playback.pause()
                override fun onStop() = state.playback.pause()
                override fun onSkipToNext() = state.playback.next()
                override fun onSkipToPrevious() = state.playback.previous()
                override fun onSeekTo(pos: Long) = state.playback.seek(pos.coerceAtLeast(0).toULong())
            })
            isActive = true
        }
        registerReceiver(noisy, IntentFilter(AudioManager.ACTION_AUDIO_BECOMING_NOISY))
        observer = state.scope.launch {
            snapshotFlow {
                val b = state.playback.bar
                listOf(b.trackIndex, b.title, b.artist, b.album, b.isPlaying, b.durationMs, state.playback.seekEpoch)
            }.collect { refresh() }
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        // startForegroundService() demands startForeground() within seconds, whatever happens next.
        refresh()
        MediaButtonReceiver.handleIntent(session, intent)
        return START_NOT_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onTaskRemoved(rootIntent: Intent?) {
        // Playback continues in the background; nothing to do.
    }

    override fun onDestroy() {
        observer?.cancel()
        coverJob?.cancel()
        abandonFocus()
        runCatching { unregisterReceiver(noisy) }
        session.isActive = false
        session.release()
        super.onDestroy()
    }

    private fun refresh() {
        val bar = state.playback.bar
        val hasTrack = bar.trackIndex >= 0 && bar.title.isNotEmpty() && bar.title != "No Track"
        if (!hasTrack) {
            // Queue cleared: tear everything down.
            session.setPlaybackState(PlaybackStateCompat.Builder().setState(PlaybackStateCompat.STATE_STOPPED, 0, 0f).build())
            trackKey = ""
            cover = null
            abandonFocus()
            if (!foreground) startForegroundNow(buildNotification(false))
            ServiceCompat.stopForeground(this, ServiceCompat.STOP_FOREGROUND_REMOVE)
            foreground = false
            stopSelf()
            return
        }

        val key = "${bar.trackIndex}|${bar.title}|${bar.artist}|${bar.album}"
        if (key != trackKey) {
            trackKey = key
            cover = null
            loadCover(key)
        }
        session.setMetadata(
            MediaMetadataCompat.Builder()
                .putString(MediaMetadataCompat.METADATA_KEY_TITLE, bar.title)
                .putString(MediaMetadataCompat.METADATA_KEY_ARTIST, bar.artist)
                .putString(MediaMetadataCompat.METADATA_KEY_ALBUM, bar.album)
                .putLong(MediaMetadataCompat.METADATA_KEY_DURATION, bar.durationMs.toLong())
                .putBitmap(MediaMetadataCompat.METADATA_KEY_ALBUM_ART, cover)
                .build()
        )
        val actions = PlaybackStateCompat.ACTION_PLAY or PlaybackStateCompat.ACTION_PAUSE or PlaybackStateCompat.ACTION_PLAY_PAUSE or
            PlaybackStateCompat.ACTION_SKIP_TO_NEXT or PlaybackStateCompat.ACTION_SKIP_TO_PREVIOUS or PlaybackStateCompat.ACTION_SEEK_TO or
            PlaybackStateCompat.ACTION_STOP
        session.setPlaybackState(
            PlaybackStateCompat.Builder()
                .setActions(actions)
                .setState(
                    if (bar.isPlaying) PlaybackStateCompat.STATE_PLAYING else PlaybackStateCompat.STATE_PAUSED,
                    state.playback.clock.positionMs.toLong(), if (bar.isPlaying) 1f else 0f,
                )
                .build()
        )

        val notification = buildNotification(true)
        if (bar.isPlaying) {
            requestFocus()
            startForegroundNow(notification)
        } else {
            // Paused: drop foreground status but keep the notification so controls stay.
            if (foreground) ServiceCompat.stopForeground(this, ServiceCompat.STOP_FOREGROUND_DETACH)
            foreground = false
            getSystemService(NotificationManager::class.java).notify(ID, notification)
            if (!pausedByFocusLoss) abandonFocus()
        }
    }

    private fun startForegroundNow(n: Notification) {
        val type = if (Build.VERSION.SDK_INT >= 29) ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK else 0
        ServiceCompat.startForeground(this, ID, n, type)
        foreground = true
    }

    private fun loadCover(key: String) {
        coverJob?.cancel()
        coverJob = state.scope.launch {
            val data = withContext(Dispatchers.Default) { state.engine?.artworkLoadCurrentThumbnail(400u) } ?: return@launch
            val bmp = withContext(Dispatchers.Default) { BitmapFactory.decodeByteArray(data, 0, data.size) } ?: return@launch
            if (key == trackKey) {
                cover = bmp
                refresh()
            }
        }
    }

    private fun buildNotification(hasTrack: Boolean): Notification {
        val bar = state.playback.bar
        fun action(icon: Int, title: String, code: Long) =
            NotificationCompat.Action(icon, title, MediaButtonReceiver.buildMediaButtonPendingIntent(this, code))
        val open = PendingIntent.getActivity(this, 0, Intent(this, MainActivity::class.java), PendingIntent.FLAG_IMMUTABLE)
        return NotificationCompat.Builder(this, CHANNEL)
            .setSmallIcon(android.R.drawable.ic_media_play)
            .setContentTitle(if (hasTrack) bar.title else "Fluyer")
            .setContentText(bar.artist)
            .setLargeIcon(cover)
            .setContentIntent(open)
            .setOnlyAlertOnce(true)
            .setOngoing(bar.isPlaying)
            .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
            .addAction(action(android.R.drawable.ic_media_previous, "Previous", PlaybackStateCompat.ACTION_SKIP_TO_PREVIOUS))
            .addAction(
                if (bar.isPlaying) action(android.R.drawable.ic_media_pause, "Pause", PlaybackStateCompat.ACTION_PAUSE)
                else action(android.R.drawable.ic_media_play, "Play", PlaybackStateCompat.ACTION_PLAY)
            )
            .addAction(action(android.R.drawable.ic_media_next, "Next", PlaybackStateCompat.ACTION_SKIP_TO_NEXT))
            .setStyle(MediaStyle().setMediaSession(session.sessionToken).setShowActionsInCompactView(0, 1, 2))
            .build()
    }

    // Audio focus: pause on loss, resume after a transient loss. ponytail: no ducking.
    private fun requestFocus() {
        if (focus != null) return
        val request = AudioFocusRequestCompat.Builder(AudioManagerCompat.AUDIOFOCUS_GAIN)
            .setAudioAttributes(
                AudioAttributesCompat.Builder()
                    .setUsage(AudioAttributesCompat.USAGE_MEDIA)
                    .setContentType(AudioAttributesCompat.CONTENT_TYPE_MUSIC)
                    .build()
            )
            .setOnAudioFocusChangeListener { change ->
                when (change) {
                    AudioManager.AUDIOFOCUS_LOSS -> { pausedByFocusLoss = false; state.playback.pause(); abandonFocus() }
                    AudioManager.AUDIOFOCUS_LOSS_TRANSIENT, AudioManager.AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK -> {
                        if (state.playback.bar.isPlaying) { pausedByFocusLoss = true; state.playback.pause() }
                    }
                    AudioManager.AUDIOFOCUS_GAIN -> if (pausedByFocusLoss) { pausedByFocusLoss = false; state.playback.play() }
                }
            }
            .build()
        focus = request
        AudioManagerCompat.requestAudioFocus(audio, request)
    }

    private fun abandonFocus() {
        focus?.let { AudioManagerCompat.abandonAudioFocusRequest(audio, it) }
        focus = null
        pausedByFocusLoss = false
    }

    private companion object {
        const val CHANNEL = "playback"
        const val ID = 1
    }
}
