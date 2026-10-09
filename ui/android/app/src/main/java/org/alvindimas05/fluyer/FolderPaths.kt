package org.alvindimas05.fluyer

import android.content.Context
import android.net.Uri
import android.os.Environment
import android.os.storage.StorageManager
import android.provider.DocumentsContract
import java.io.File

/**
 * Maps a SAF tree URI to a filesystem path (the core reads plain paths).
 * Port of tauri-plugin-fluyer FileUtil.kt. ponytail: Android 11+ only (minSdk 26 devices
 * below 30 use the primary volume or reflection-free best effort via getExternalStorageDirectory).
 */
object FolderPaths {
    fun fromTreeUri(context: Context, tree: Uri): String? {
        val docId = runCatching { DocumentsContract.getTreeDocumentId(tree) }.getOrNull() ?: return null
        val volumeId = docId.substringBefore(':')
        val relative = if (':' in docId) docId.substringAfter(':') else ""
        val root = volumePath(context, volumeId) ?: return null
        return File(root, relative).path.trimEnd('/').ifEmpty { "/" }
    }

    private fun volumePath(context: Context, volumeId: String): String? {
        if (volumeId == "primary") return Environment.getExternalStorageDirectory().path
        if (android.os.Build.VERSION.SDK_INT < 30) return null
        val sm = context.getSystemService(Context.STORAGE_SERVICE) as StorageManager
        return sm.storageVolumes.firstOrNull { it.uuid == volumeId }?.directory?.path
    }
}
