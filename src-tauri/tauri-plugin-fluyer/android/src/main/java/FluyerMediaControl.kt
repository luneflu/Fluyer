package org.alvindimas05.fluyerplugin

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.graphics.BitmapFactory
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.support.v4.media.MediaMetadataCompat
import android.support.v4.media.session.MediaSessionCompat
import android.support.v4.media.session.PlaybackStateCompat
import android.util.Log
import androidx.core.app.NotificationCompat

class FluyerMediaControl(private val context: Context, private val onAction: (String) -> Unit) {
    private val mediaSession: MediaSessionCompat = MediaSessionCompat(context, "FluyerMediaSession")
    private val notificationManager =
            context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    private val channelId = "fluyer_media_control"
    private val handler = Handler(Looper.getMainLooper())
    private var isFirst = false
    private var isLast = false
    private var isStopped = false

    init {
        createNotificationChannel()

        mediaSession.setCallback(
                object : MediaSessionCompat.Callback() {
                    override fun onPlay() {
                        onAction("play")
                        Log.d(LOG_TAG, "MediaSession onPlay called")
                        updatePlaybackState(PlaybackStateCompat.STATE_PLAYING, getCurrentPosition())
                    }

                    override fun onPause() {
                        onAction("pause")
                        Log.d(LOG_TAG, "MediaSession onPause called")
                        updatePlaybackState(PlaybackStateCompat.STATE_PAUSED, getCurrentPosition())
                    }

                    override fun onSkipToNext() {
                        onAction("next")
                        Log.d(LOG_TAG, "MediaSession onSkipToNext called")
                    }

                    override fun onSkipToPrevious() {
                        onAction("previous")
                        Log.d(LOG_TAG, "MediaSession onSkipToPrevious called")
                    }

                    override fun onSeekTo(pos: Long) {
                        onAction("seek:$pos")
                        Log.d(LOG_TAG, "MediaSession onSeekTo called: $pos")
                    }

                    override fun onStop() {
                        onAction("stop")
                        isStopped = true
                        notificationManager.cancel(1)
                        Log.d(LOG_TAG, "MediaSession onStop called")
                        updatePlaybackState(PlaybackStateCompat.STATE_STOPPED, getCurrentPosition())
                    }
                }
        )

        mediaSession.isActive = true
        updatePlaybackState(PlaybackStateCompat.STATE_NONE)

        // Set up the BroadcastReceiver callback to forward to MediaSession
        // Use handler.post to avoid blocking the BroadcastReceiver and causing ANR
        MediaControlReceiver.pendingCallback = { action ->
            handler.post {
                when (action) {
                    MediaControlReceiver.ACTION_PLAY ->
                            mediaSession.controller?.transportControls?.play()
                    MediaControlReceiver.ACTION_PAUSE ->
                            mediaSession.controller?.transportControls?.pause()
                    MediaControlReceiver.ACTION_PREVIOUS ->
                            mediaSession.controller?.transportControls?.skipToPrevious()
                    MediaControlReceiver.ACTION_NEXT ->
                            mediaSession.controller?.transportControls?.skipToNext()
                    MediaControlReceiver.ACTION_STOP -> {
                        mediaSession.controller.transportControls.stop()
                    }
                }
            }
        }
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val name = "Media Control"
            val descriptionText = "Control music playback"
            val importance = NotificationManager.IMPORTANCE_LOW
            val channel =
                    NotificationChannel(channelId, name, importance).apply {
                        description = descriptionText
                        setShowBadge(false)
                        lockscreenVisibility = Notification.VISIBILITY_PUBLIC
                        setSound(null, null) // Silent notification
                        enableVibration(false)
                    }
            notificationManager.createNotificationChannel(channel)
        }
    }

    private fun getCurrentPosition(): Long {
        mediaSession.controller.playbackState?.let {
            var position = it.position
            if (it.state == PlaybackStateCompat.STATE_PLAYING) {
                val timeDelta = android.os.SystemClock.elapsedRealtime() - it.lastPositionUpdateTime
                position += (timeDelta * it.playbackSpeed).toLong()
            }
            return position
        }
        return PlaybackStateCompat.PLAYBACK_POSITION_UNKNOWN
    }

    private fun updatePlaybackState(
            state: Int,
            position: Long = PlaybackStateCompat.PLAYBACK_POSITION_UNKNOWN
    ) {
        val playbackState =
                PlaybackStateCompat.Builder()
                        .setActions(
                                PlaybackStateCompat.ACTION_PLAY or
                                        PlaybackStateCompat.ACTION_PAUSE or
                                        (if (isLast) 0
                                        else PlaybackStateCompat.ACTION_SKIP_TO_NEXT) or
                                        (if (isFirst) 0
                                        else PlaybackStateCompat.ACTION_SKIP_TO_PREVIOUS) or
                                        PlaybackStateCompat.ACTION_SEEK_TO
                        )
                        .setState(state, position, 1.0f, android.os.SystemClock.elapsedRealtime())
                        .build()
        mediaSession.setPlaybackState(playbackState)
    }

    fun updateMetadata(
            title: String,
            artist: String,
            album: String,
            duration: Long,
            artworkPath: String?,
            isPlaying: Boolean,
            isFirst: Boolean,
            isLast: Boolean
    ) {
        isStopped = false
        val builder =
                MediaMetadataCompat.Builder()
                        .putString(MediaMetadataCompat.METADATA_KEY_TITLE, title)
                        .putString(MediaMetadataCompat.METADATA_KEY_ARTIST, artist)
                        .putString(MediaMetadataCompat.METADATA_KEY_ALBUM, album)
                        .putLong(MediaMetadataCompat.METADATA_KEY_DURATION, duration)

        if (artworkPath != null) {
            try {
                val bitmap = BitmapFactory.decodeFile(artworkPath)
                if (bitmap != null) {
                    builder.putBitmap(MediaMetadataCompat.METADATA_KEY_ALBUM_ART, bitmap)
                }
            } catch (e: Exception) {
                // Ignore error loading artwork
            }
        }

        this.isFirst = isFirst
        this.isLast = isLast

        mediaSession.setMetadata(builder.build())

        // Also update playback state so session is in sync
        val state =
                if (isPlaying) PlaybackStateCompat.STATE_PLAYING
                else PlaybackStateCompat.STATE_PAUSED
        updatePlaybackState(state)

        showNotification(title, artist, artworkPath, isPlaying)
    }

    fun setPlaybackState(isPlaying: Boolean) {
        val state =
                if (isPlaying) PlaybackStateCompat.STATE_PLAYING
                else PlaybackStateCompat.STATE_PAUSED
        updatePlaybackState(state)

        // Refresh notification with new state (to show correct play/pause button)
        val metadata = mediaSession.controller.metadata
        if (metadata != null) {
            val title = metadata.getString(MediaMetadataCompat.METADATA_KEY_TITLE) ?: "Unknown"
            val artist = metadata.getString(MediaMetadataCompat.METADATA_KEY_ARTIST) ?: "Unknown"
            // We don't easily have the path here, might need to store it or extract from metadata
            // if possible (bitmap is there)
            // For now, simpler to just rely on next update call or refactor to store current state.
            // Actually, for just play/pause update, we can rebuild notification.
            // But we need the artwork again?
            // Better to just update status and let logic call updateMetadata if artwork changes.
            // Or we can cache the last artwork path in this class.
        }
    }

    // Store last artwork path for creating notification without reloading everything
    private var lastArtworkPath: String? = null

    fun updateState(isPlaying: Boolean, position: Long) {
        val state =
                if (isPlaying) PlaybackStateCompat.STATE_PLAYING
                else PlaybackStateCompat.STATE_PAUSED

        if (isPlaying) {
            isStopped = false
        }

        Log.d(LOG_TAG, "updateState called: isPlaying=$isPlaying, position=$position")

        val playbackState =
                PlaybackStateCompat.Builder()
                        .setActions(
                                PlaybackStateCompat.ACTION_PLAY or
                                        PlaybackStateCompat.ACTION_PAUSE or
                                        (if (isLast) 0
                                        else PlaybackStateCompat.ACTION_SKIP_TO_NEXT) or
                                        (if (isFirst) 0
                                        else PlaybackStateCompat.ACTION_SKIP_TO_PREVIOUS) or
                                        PlaybackStateCompat.ACTION_SEEK_TO
                        )
                        .setState(state, position, 1.0f, android.os.SystemClock.elapsedRealtime())
                        .build()
        mediaSession.setPlaybackState(playbackState)

        if (!isStopped) {
            // Update notification in-place (no cancel needed, notify with same ID updates smoothly)
            val metadata = mediaSession.controller.metadata
            if (metadata != null) {
                val title = metadata.getString(MediaMetadataCompat.METADATA_KEY_TITLE) ?: ""
                val artist = metadata.getString(MediaMetadataCompat.METADATA_KEY_ARTIST) ?: ""
                showNotification(title, artist, lastArtworkPath, isPlaying)
            }
        }
    }

    private fun showNotification(
            title: String,
            artist: String,
            artworkPath: String?,
            isPlaying: Boolean
    ) {
        lastArtworkPath = artworkPath

        val controller = mediaSession.controller
        val mediaMetadata = controller.metadata

        val builder =
            NotificationCompat.Builder(context, channelId)
                .setContentTitle(title)
                .setContentText(artist)
                .setSubText(
                    mediaMetadata.getString(MediaMetadataCompat.METADATA_KEY_ALBUM)
                )
                .setSmallIcon(android.R.drawable.ic_media_play) // TODO: Use app icon
                .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
                .setDeleteIntent(createPendingIntent(MediaControlReceiver.ACTION_STOP))
                .setOngoing(false)

        val compactActionIndices = mutableListOf<Int>()
        var currentIndex = 0

        // Actions
        if (!isFirst) {
            builder.addAction(
                NotificationCompat.Action(
                    android.R.drawable.ic_media_previous,
                    "Previous",
                    createPendingIntent(MediaControlReceiver.ACTION_PREVIOUS)
                )
            )
            compactActionIndices.add(currentIndex++)
        }

        if (isPlaying) {
            builder.addAction(
                NotificationCompat.Action(
                    android.R.drawable.ic_media_pause,
                    "Pause",
                    createPendingIntent(MediaControlReceiver.ACTION_PAUSE)
                )
            )
            compactActionIndices.add(currentIndex++)
        } else {
            builder.addAction(
                NotificationCompat.Action(
                    android.R.drawable.ic_media_play,
                    "Play",
                    createPendingIntent(MediaControlReceiver.ACTION_PLAY)
                )
            )
            compactActionIndices.add(currentIndex++)
        }

        if (!isLast) {
            builder.addAction(
                NotificationCompat.Action(
                    android.R.drawable.ic_media_next,
                    "Next",
                    createPendingIntent(MediaControlReceiver.ACTION_NEXT)
                )
            )
            compactActionIndices.add(currentIndex++)
        }

        builder.setStyle(
            androidx.media.app.NotificationCompat.MediaStyle()
                .setMediaSession(mediaSession.sessionToken)
                .setShowActionsInCompactView(*compactActionIndices.toIntArray())
        )

        // Large Icon (Artwork)
        if (artworkPath != null) {
            try {
                val bitmap = BitmapFactory.decodeFile(artworkPath)
                if (bitmap != null) {
                    builder.setLargeIcon(bitmap)
                }
            } catch (e: Exception) {}
        }

        // Intent to open app when clicking notification
        val intent = context.packageManager.getLaunchIntentForPackage(context.packageName)
        if (intent != null) {
            val pendingIntent =
                    PendingIntent.getActivity(
                            context,
                            0,
                            intent,
                            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
                    )
            builder.setContentIntent(pendingIntent)
        }

        notificationManager.notify(1, builder.build())
    }

    private fun createPendingIntent(action: String): PendingIntent {
        val intent = Intent(action).apply { setPackage(context.packageName) }
        return PendingIntent.getBroadcast(
                context,
                action.hashCode(),
                intent,
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )
    }

    fun release() {
        MediaControlReceiver.pendingCallback = null
        mediaSession.isActive = false
        mediaSession.release()
        notificationManager.cancel(1)
    }
}
