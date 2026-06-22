/// Error type definitions.
///
/// This enum is annotated with [`#[non_exhaustive]`]; new variants may be
/// added in future minor releases.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Invalid idiom.
    InvalidIdiom(String),
    /// Invalid mnemonic length.
    InvalidLength(usize),
    /// Checksum mismatch.
    ChecksumMismatch,
    /// Invalid entropy length.
    InvalidEntropyLength(usize),
    /// Invalid index.
    InvalidIndex(usize),
    /// Random number generation failed.
    #[cfg(feature = "rand")]
    RandError(getrandom::Error),
    /// Random number source unavailable (rand feature not enabled).
    #[cfg(not(feature = "rand"))]
    RandUnavailable,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidIdiom(idiom) => write!(f, "Invalid idiom: {}", idiom),
            Error::InvalidLength(len) => write!(f, "Invalid mnemonic length: {}", len),
            Error::ChecksumMismatch => write!(f, "Checksum mismatch"),
            Error::InvalidEntropyLength(len) => write!(f, "Invalid entropy length: {} bits", len),
            Error::InvalidIndex(idx) => write!(f, "Invalid idiom index: {}", idx),
            #[cfg(feature = "rand")]
            Error::RandError(e) => write!(f, "Random generation error: {}", e),
            #[cfg(not(feature = "rand"))]
            Error::RandUnavailable => write!(
                f,
                "Random number generator unavailable"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

// In no_std environments, also implement core::error::Error (stabilized in Rust 1.81+),
// for compatibility with embedded / hardware wallet scenarios.
#[cfg(not(feature = "std"))]
impl core::error::Error for Error {}

#[cfg(feature = "rand")]
impl From<getrandom::Error> for Error {
    fn from(e: getrandom::Error) -> Self {
        Error::RandError(e)
    }
}

/// Result type alias.
pub type Result<T> = core::result::Result<T, Error>;