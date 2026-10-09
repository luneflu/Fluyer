package org.alvindimas05.fluyer

import android.Manifest
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat
import org.alvindimas05.fluyer.state.AppState
import org.alvindimas05.fluyer.ui.FluyerApp

class MainActivity : ComponentActivity() {
    private val app get() = application as FluyerApplication
    private val state: AppState get() = app.state

    /** Runs after the audio permission is granted (immediately if it already is). */
    private var afterPermission: (() -> Unit)? = null

    private val permissions = registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) {
        if (hasAudioPermission()) afterPermission?.invoke()
        afterPermission = null
    }

    private val pickFolder = registerForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri: Uri? ->
        val path = uri?.let { FolderPaths.fromTreeUri(this, it) }
        if (uri != null && path == null) state.toast.show("Cannot use this folder")
        if (path != null) withAudioPermission { state.scanFolders(listOf(path)) }
    }

    private fun hasAudioPermission() = ContextCompat.checkSelfPermission(this, audioPermission) == PackageManager.PERMISSION_GRANTED

    private fun withAudioPermission(action: () -> Unit) {
        if (hasAudioPermission()) return action()
        afterPermission = action
        val wanted = mutableListOf(audioPermission)
        if (Build.VERSION.SDK_INT >= 33) wanted += Manifest.permission.POST_NOTIFICATIONS
        permissions.launch(wanted.toTypedArray())
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        if (!app.scannedSavedFolders) {
            app.scannedSavedFolders = true
            withAudioPermission { state.scanSavedFolders() }
        }
        setContent {
            FluyerApp(
                state,
                onAddFolder = { pickFolder.launch(null) },
                onRescan = { withAudioPermission { state.scanSavedFolders() } },
            )
        }
    }

    override fun onStop() {
        state.saveSession()
        super.onStop()
    }

    private companion object {
        val audioPermission = if (Build.VERSION.SDK_INT >= 33) Manifest.permission.READ_MEDIA_AUDIO else Manifest.permission.READ_EXTERNAL_STORAGE
    }
}
