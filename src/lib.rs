#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

//! # fci4096
//!
//! A BIP39-inspired mnemonic implementation using 4096 Chinese four-character idioms
//! as the wordlist.
//!
//! Each idiom encodes 12 bits of information (2^12 = 4096), providing higher
//! information density compared to BIP39's 2048-word list (11 bits/word) for the
//! same number of words. The checksum is 1/8 of the entropy length.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use fci4096::{generate, from_phrase, IdiomMnemonicSize};
//!
//! // Generate a 12-idiom mnemonic (128 bits of entropy)
//! let mnemonic = generate(IdiomMnemonicSize::Idioms12).unwrap();
//! println!("{}", mnemonic.phrase());
//!
//! // Derive the seed
//! let seed = mnemonic.to_seed("my_passphrase");
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `std`   | Yes     | Standard library support |
//! | `rand`  | Yes     | Random number generation (requires `getrandom`) |
//! | `serde` | No      | Serialize/Deserialize support |
//! | `wasm`  | No      | WASM bindings (includes `rand` automatically) |

#[cfg(feature = "std")]
extern crate std;

/// Error type definitions
pub mod error;
/// Entropy encoding/decoding (bit operations and checksum calculation)
pub mod entropy;
/// Idiom wordlist and lookup
pub mod idiom_list;
/// Core mnemonic types
pub mod mnemonic;
/// Seed generation (PBKDF2-SHA512)
pub mod seed;

/// WASM bindings (requires the `wasm` feature)
#[cfg(feature = "wasm")]
pub mod wasm;

pub use crate::error::{Error, Result};
pub use crate::idiom_list::ChineseIdiomList;
pub use crate::mnemonic::{IdiomMnemonic, IdiomMnemonicSize};
pub use crate::seed::PBKDF2_ITERATIONS;

/// Generate a random mnemonic.
///
/// Uses the operating system's cryptographically secure random source.
/// Requires the `rand` feature.
///
/// # Example
///
/// ```rust,no_run
/// use fci4096::{generate, IdiomMnemonicSize};
///
/// let mnemonic = generate(IdiomMnemonicSize::Idioms12).unwrap();
/// assert_eq!(mnemonic.idioms().len(), 12);
/// ```
#[cfg(feature = "rand")]
pub fn generate(size: IdiomMnemonicSize) -> Result<IdiomMnemonic> {
    IdiomMnemonic::generate(size)
}

/// Create a mnemonic from entropy bytes.
///
/// Entropy must be 16/20/24/28/32 bytes (128/160/192/224/256 bits).
///
/// # Example
///
/// ```rust
/// use fci4096::from_entropy;
///
/// let entropy = [0x00u8; 16]; // 128 bits of entropy
/// let mnemonic = from_entropy(&entropy).unwrap();
/// assert_eq!(mnemonic.idioms().len(), 12);
/// ```
pub fn from_entropy(entropy: &[u8]) -> Result<IdiomMnemonic> {
    IdiomMnemonic::from_entropy(entropy)
}

/// Parse a mnemonic from an idiom phrase (with checksum validation).
///
/// Idioms are separated by spaces (full-width/half-width spaces and tabs are accepted).
/// Returns `Error::ChecksumMismatch` if the checksum does not match.
///
/// # Example
///
/// ```rust
/// use fci4096::{from_entropy, from_phrase};
///
/// let entropy = [0x00u8; 16];
/// let mnemonic = from_entropy(&entropy).unwrap();
/// let phrase = mnemonic.phrase();
/// let recovered = from_phrase(&phrase).unwrap();
/// assert_eq!(mnemonic, recovered);
/// ```
pub fn from_phrase(phrase: &str) -> Result<IdiomMnemonic> {
    IdiomMnemonic::from_phrase(phrase)
}

/// Validate a mnemonic phrase.
///
/// Checks whether all idioms exist in the wordlist and the checksum matches.
///
/// # Example
///
/// ```rust
/// use fci4096::{generate, validate, IdiomMnemonicSize};
///
/// let mnemonic = generate(IdiomMnemonicSize::Idioms12).unwrap();
/// assert!(validate(&mnemonic.phrase()));
/// assert!(!validate("invalid phrase"));
/// ```
pub fn validate(phrase: &str) -> bool {
    IdiomMnemonic::validate(phrase)
}

/// Look up an idiom's index via binary search.
///
/// Uses binary search over a Unicode codepoint-sorted lookup array.
/// Time complexity O(log N), N = 4096.
///
/// # Example
///
/// ```rust
/// use fci4096::idiom_to_index;
///
/// if let Some(idx) = idiom_to_index("爱屋及乌") {
///     println!("Idiom index: {}", idx);
/// }
/// ```
pub fn idiom_to_index(idiom: &str) -> Option<usize> {
    ChineseIdiomList::idiom_to_index(idiom)
}

/// Look up an idiom by its index.
///
/// # Example
///
/// ```rust
/// use fci4096::index_to_idiom;
///
/// if let Some(idiom) = index_to_idiom(0) {
///     println!("First idiom in wordlist: {}", idiom);
/// }
/// ```
pub fn index_to_idiom(idx: usize) -> Option<&'static str> {
    ChineseIdiomList::index_to_idiom(idx)
}

/// Get the pinyin of an idiom by index.
///
/// # Example
///
/// ```rust
/// use fci4096::get_pinyin;
///
/// if let Some(pinyin) = get_pinyin(0) {
///     println!("Pinyin: {}", pinyin);
/// }
/// ```
pub fn get_pinyin(idx: usize) -> Option<&'static str> {
    ChineseIdiomList::get_pinyin(idx)
}

/// Total number of idioms (fixed at 4096)
pub const IDIOM_COUNT: usize = 4096;