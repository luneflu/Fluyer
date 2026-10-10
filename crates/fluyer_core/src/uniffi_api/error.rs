// Field is `reason`, not `message`: Kotlin bindings map errors to exceptions and a
// `message` field collides with `Throwable.message` (UniFFI 0.28).
#[derive(Debug, uniffi::Error)]
pub enum FluyerError {
    InitFailed { reason: String },
}

impl std::fmt::Display for FluyerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FluyerError::InitFailed { reason } => write!(f, "Initialization failed: {}", reason),
        }
    }
}

impl std::error::Error for FluyerError {}
