package org.alvindimas05.fluyer.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/** Transient status messages from the core. A new toast cancels the previous timer so it cannot clear its successor. */
class ToastState(private val scope: CoroutineScope) {
    var message by mutableStateOf<String?>(null); private set
    private var dismissal: Job? = null

    fun show(message: String) {
        this.message = message
        dismissal?.cancel()
        dismissal = scope.launch {
            delay(3000)
            this@ToastState.message = null
        }
    }

    fun dismiss() {
        dismissal?.cancel()
        dismissal = null
        message = null
    }
}
