extension FloatingPoint {
    /// Confine to `limits`.
    ///
    /// ponytail: NaN collapses to `limits.lowerBound` instead of propagating.
    /// `Swift.min`/`Swift.max` pass NaN straight through, and every caller feeds the
    /// result into an `Int`/`UInt64` conversion (or a SwiftUI frame width) that traps
    /// on it — the zero-width drag case is a real way to get there. Infinities are left
    /// to `min`/`max`, which pin them to the correct end of the range.
    func clamped(to limits: ClosedRange<Self>) -> Self {
        guard !isNaN else { return limits.lowerBound }
        return Swift.min(Swift.max(self, limits.lowerBound), limits.upperBound)
    }
}
