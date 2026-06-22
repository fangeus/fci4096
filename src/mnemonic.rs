use crate::entropy::{
    entropy_to_indices, generate_entropy, indices_to_entropy, indices_to_entropy_inner,
    validate_mnemonic_length,
};
use crate::error::{Error, Result};
use crate::idiom_list::ChineseIdiomList;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Mnemonic length variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum IdiomMnemonicSize {
    /// 12 idioms, 128 bits of entropy + 16-bit checksum
    Idioms12,
    /// 15 idioms, 160 bits of entropy + 20-bit checksum
    Idioms15,
    /// 18 idioms, 192 bits of entropy + 24-bit checksum
    Idioms18,
    /// 21 idioms, 224 bits of entropy + 28-bit checksum
    Idioms21,
    /// 24 idioms, 256 bits of entropy + 32-bit checksum
    Idioms24,
}

impl IdiomMnemonicSize {
    /// The corresponding entropy size in bits.
    pub fn entropy_bits(&self) -> usize {
        match self {
            IdiomMnemonicSize::Idioms12 => 128,
            IdiomMnemonicSize::Idioms15 => 160,
            IdiomMnemonicSize::Idioms18 => 192,
            IdiomMnemonicSize::Idioms21 => 224,
            IdiomMnemonicSize::Idioms24 => 256,
        }
    }

    /// The corresponding idiom count.
    pub fn idiom_count(&self) -> usize {
        match self {
            IdiomMnemonicSize::Idioms12 => 12,
            IdiomMnemonicSize::Idioms15 => 15,
            IdiomMnemonicSize::Idioms18 => 18,
            IdiomMnemonicSize::Idioms21 => 21,
            IdiomMnemonicSize::Idioms24 => 24,
        }
    }
}

/// An idiom-based mnemonic.
///
/// Internally stores idiom indices as `Vec<u16>` (2 bytes per index, replacing
/// the previous `Vec<String>` heap allocations), significantly reducing memory
/// usage and allocation count. Provides `phrase()`, `idioms()`, `to_entropy()`,
/// and other methods.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IdiomMnemonic {
    indices: Vec<u16>,
}

impl IdiomMnemonic {
    /// Generate a random mnemonic.
    ///
    /// Uses the operating system's cryptographically secure random source.
    /// Requires the `rand` feature.
    #[cfg(feature = "rand")]
    pub fn generate(size: IdiomMnemonicSize) -> Result<Self> {
        let entropy = generate_entropy(size.entropy_bits())?;
        Self::from_entropy(&entropy)
    }

    /// Create a mnemonic from entropy bytes.
    ///
    /// Entropy must be 16/20/24/28/32 bytes (128/160/192/224/256 bits).
    pub fn from_entropy(entropy: &[u8]) -> Result<Self> {
        let entropy_bits = entropy.len() * 8;
        if !matches!(entropy_bits, 128 | 160 | 192 | 224 | 256) {
            return Err(Error::InvalidEntropyLength(entropy_bits));
        }

        let indices = entropy_to_indices(entropy);
        let indices_u16: Vec<u16> = indices
            .iter()
            .map(|&idx| {
                idx.try_into()
                    .map_err(|_| Error::InvalidIndex(idx))
            })
            .collect::<Result<_>>()?;

        Ok(IdiomMnemonic {
            indices: indices_u16,
        })
    }

    /// Parse a mnemonic from an idiom phrase (with checksum validation).
    ///
    /// Idioms are separated by spaces (full-width/half-width and tabs are supported).
    /// Automatically validates the checksum after parsing; returns
    /// `Error::ChecksumMismatch` on failure.
    pub fn from_phrase(phrase: &str) -> Result<Self> {
        // Split by spaces (half-width, full-width, tab) without intermediate String allocation
        let idiom_strings: Vec<&str> = phrase
            .split(|c: char| c == ' ' || c == '\u{3000}' || c == '\t')
            .filter(|s| !s.is_empty())
            .collect();

        if !validate_mnemonic_length(idiom_strings.len()) {
            return Err(Error::InvalidLength(idiom_strings.len()));
        }

        let mut indices = Vec::with_capacity(idiom_strings.len());
        for idiom in &idiom_strings {
            let idx = ChineseIdiomList::idiom_to_index(idiom)
                .ok_or_else(|| Error::InvalidIdiom(idiom.to_string()))?;
            indices.push(idx);
        }

        // Use the inner path to skip redundant index range check (already validated
        // in idiom_to_index)
        indices_to_entropy_inner(&indices)?;

        let indices_u16: Vec<u16> = indices
            .into_iter()
            .map(|idx| idx as u16)
            .collect();

        Ok(IdiomMnemonic {
            indices: indices_u16,
        })
    }

    /// Recover the entropy from this mnemonic.
    pub fn to_entropy(&self) -> Result<Vec<u8>> {
        let indices: Vec<usize> = self.indices.iter().map(|&i| i as usize).collect();
        indices_to_entropy(&indices)
    }

    /// Derive a 64-byte seed from this mnemonic (BIP39-style).
    ///
    /// Uses PBKDF2-SHA512 with the default 4096 iterations.
    /// Both the mnemonic and passphrase are NFKD-normalized.
    #[must_use]
    pub fn to_seed(&self, passphrase: &str) -> [u8; 64] {
        crate::seed::mnemonic_to_seed(&self.phrase(), passphrase)
    }

    /// Derive a 64-byte seed with a custom iteration count.
    ///
    /// See [`crate::seed::mnemonic_to_seed_with_iterations`].
    #[must_use]
    pub fn to_seed_with_iterations(&self, passphrase: &str, iterations: u32) -> [u8; 64] {
        crate::seed::mnemonic_to_seed_with_iterations(&self.phrase(), passphrase, iterations)
    }

    /// Validate a mnemonic phrase.
    pub fn validate(phrase: &str) -> bool {
        Self::from_phrase(phrase).is_ok()
    }

    /// Look up an idiom's index.
    pub fn idiom_to_index(idiom: &str) -> Option<usize> {
        ChineseIdiomList::idiom_to_index(idiom)
    }

    /// Look up an idiom by its index.
    pub fn index_to_idiom(idx: usize) -> Option<&'static str> {
        ChineseIdiomList::index_to_idiom(idx)
    }

    /// Get all idioms as `&'static str` references (zero heap cost per idiom, but
    /// collected into a `Vec`).
    pub fn idioms(&self) -> Vec<&'static str> {
        self.indices
            .iter()
            .map(|&i| ChineseIdiomList::index_to_idiom(i as usize).unwrap())
            .collect()
    }

    /// Get the mnemonic phrase (space-separated idiom string).
    ///
    /// Single pass, single allocation; no intermediate `Vec`.
    pub fn phrase(&self) -> String {
        let mut s = String::with_capacity(self.indices.len() * 8); // ~8 bytes per idiom
        for (i, &idx) in self.indices.iter().enumerate() {
            if i > 0 {
                s.push(' ');
            }
            // Index is validated at construction time; unwrap is safe
            s.push_str(ChineseIdiomList::index_to_idiom(idx as usize).unwrap());
        }
        s
    }

    /// Get the raw index slice (for advanced use such as serialization).
    pub fn raw_indices(&self) -> &[u16] {
        &self.indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "rand")]
    fn test_generate_idiom_mnemonic() {
        let idiom_mnemonic = IdiomMnemonic::generate(IdiomMnemonicSize::Idioms12).unwrap();
        assert_eq!(idiom_mnemonic.idioms().len(), 12);
        assert!(IdiomMnemonic::validate(&idiom_mnemonic.phrase()));
    }

    #[test]
    fn test_from_entropy() {
        let entropy = [0x00u8; 16];
        let idiom_mnemonic = IdiomMnemonic::from_entropy(&entropy).unwrap();
        assert_eq!(idiom_mnemonic.idioms().len(), 12);
    }

    #[test]
    fn test_from_phrase() {
        let entropy = [0x00u8; 16];
        let idiom_mnemonic = IdiomMnemonic::from_entropy(&entropy).unwrap();
        let phrase = idiom_mnemonic.phrase();
        let recovered = IdiomMnemonic::from_phrase(&phrase).unwrap();
        assert_eq!(idiom_mnemonic, recovered);
    }

    #[test]
    fn test_to_entropy_roundtrip() {
        let entropy = [0x00u8; 16];
        let idiom_mnemonic = IdiomMnemonic::from_entropy(&entropy).unwrap();
        let recovered = idiom_mnemonic.to_entropy().unwrap();
        assert_eq!(entropy, recovered.as_slice());
    }

    #[test]
    fn test_validate() {
        let entropy = [0x00u8; 16];
        let idiom_mnemonic = IdiomMnemonic::from_entropy(&entropy).unwrap();
        assert!(IdiomMnemonic::validate(&idiom_mnemonic.phrase()));
        assert!(!IdiomMnemonic::validate("invalid phrase"));
    }

    #[test]
    #[cfg(feature = "rand")]
    fn test_all_sizes() {
        for size in [
            IdiomMnemonicSize::Idioms12,
            IdiomMnemonicSize::Idioms15,
            IdiomMnemonicSize::Idioms18,
            IdiomMnemonicSize::Idioms21,
            IdiomMnemonicSize::Idioms24,
        ]
        .iter()
        {
            let idiom_mnemonic = IdiomMnemonic::generate(*size).unwrap();
            assert_eq!(idiom_mnemonic.idioms().len(), size.idiom_count());
            assert!(IdiomMnemonic::validate(&idiom_mnemonic.phrase()));
        }
    }

    #[test]
    fn test_seed_roundtrip() {
        let entropy = [0x00u8; 16];
        let idiom_mnemonic = IdiomMnemonic::from_entropy(&entropy).unwrap();
        let seed = idiom_mnemonic.to_seed("");
        assert_eq!(seed.len(), 64);

        let seed_with_iter = idiom_mnemonic.to_seed_with_iterations("", 4096);
        assert_eq!(seed, seed_with_iter);
    }

    #[test]
    fn test_raw_indices() {
        let entropy = [0x00u8; 16];
        let idiom_mnemonic = IdiomMnemonic::from_entropy(&entropy).unwrap();
        let indices = idiom_mnemonic.raw_indices();
        assert_eq!(indices.len(), 12);
        for &idx in indices {
            assert!(idx < 4096);
        }
    }
}