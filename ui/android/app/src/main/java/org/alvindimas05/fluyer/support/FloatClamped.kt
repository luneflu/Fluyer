package org.alvindimas05.fluyer.support

/** Clamp a value into a range. */
fun Float.clamped(lo: Float, hi: Float): Float = if (this < lo) lo else if (this > hi) hi else this
