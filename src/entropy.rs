use crate::error::{Error, Result};
use sha2::{Digest, Sha256};

/// 4096-word list: 12 bits per word
const BITS_PER_WORD: usize = 12;

/// Valid entropy sizes in bits: 128/160/192/224/256
pub const ENTROPY_BITS_VALID: &[usize] = &[128, 160, 192, 224, 256];

/// Valid mnemonic word counts for the 4096-word list.
///
/// Each word encodes 12 bits. Checksum = entropy / 8.
///
/// Derivation:
/// - Total bits = words × 12
/// - Total bits = entropy + checksum = entropy + entropy / 8 = entropy × 9/8
/// - Therefore: words = total bits / 12 = (entropy × 9/8) / 12 = entropy × 3 / 32
///
/// | Entropy | Checksum | Total Bits | Words |
/// |---------|----------|------------|-------|
/// | 128     | 16       | 144        | 12    |
/// | 160     | 20       | 180        | 15    |
/// | 192     | 24       | 216        | 18    |
/// | 224     | 28       | 252        | 21    |
/// | 256     | 32       | 288        | 24    |
pub const MNEMONIC_WORDS_VALID: &[usize] = &[12, 15, 18, 21, 24];

/// Generate cryptographically secure random entropy of the given bit length.
///
/// Uses the operating system's random source (via `getrandom`).
/// Requires the `rand` feature. On wasm32 targets, the `getrandom/js` feature
/// is also needed.
pub fn generate_entropy(bits: usize) -> Result<Vec<u8>> {
    if !ENTROPY_BITS_VALID.contains(&bits) {
        return Err(Error::InvalidEntropyLength(bits));
    }

    let bytes = bits / 8;
    let mut entropy = vec![0u8; bytes];

    #[cfg(feature = "rand")]
    {
        getrandom::getrandom(&mut entropy)?;
    }

    #[cfg(not(feature = "rand"))]
    {
        return Err(Error::RandUnavailable);
    }

    Ok(entropy)
}

/// Compute the checksum bit count for a given entropy length.
///
/// Checksum bits = total bits - entropy bits (where total bits = words × 12).
fn checksum_bits_for_entropy(entropy_bits: usize) -> usize {
    let word_count = (entropy_bits + entropy_bits / 8) / BITS_PER_WORD;
    word_count * BITS_PER_WORD - entropy_bits
}

/// Word count to entropy bits lookup table (avoids fragile integer division).
const WORD_COUNT_TO_ENTROPY_BITS: [(usize, usize); 5] = [
    (12, 128),
    (15, 160),
    (18, 192),
    (21, 224),
    (24, 256),
];

/// Look up entropy bits for a given word count (table lookup, avoids integer division rounding).
fn entropy_bits_for_word_count(word_count: usize) -> usize {
    for &(wc, bits) in &WORD_COUNT_TO_ENTROPY_BITS {
        if wc == word_count {
            return bits;
        }
    }
    // Callers should ensure word_count is valid; this fallback is defensive only.
    word_count * BITS_PER_WORD * 8 / 9
}

/// Compute the leading N bits of the SHA-256 checksum.
pub fn calculate_checksum(entropy: &[u8], checksum_bits: usize) -> u32 {
    let hash = Sha256::digest(entropy);
    // Take the first 4 bytes of SHA-256 as a u32, then right-shift to get the desired bits.
    let full = u32::from_be_bytes(hash[..4].try_into().unwrap());
    full >> (32 - checksum_bits)
}

/// Convert entropy bytes into idiom indices.
///
/// Appends checksum bits to the entropy bitstream, then slices into 12-bit indices (0..4095).
/// Reads bits directly from entropy and checksum without allocating a temporary bit buffer.
pub fn entropy_to_indices(entropy: &[u8]) -> Vec<usize> {
    let entropy_bits = entropy.len() * 8;
    let checksum_bits = checksum_bits_for_entropy(entropy_bits);
    let checksum = calculate_checksum(entropy, checksum_bits);

    let total_bits = entropy_bits + checksum_bits;
    let word_count = total_bits / BITS_PER_WORD;
    let mut indices = Vec::with_capacity(word_count);

    for w in 0..word_count {
        let mut idx = 0usize;
        for b in 0..BITS_PER_WORD {
            let bit_pos = w * BITS_PER_WORD + b;
            let bit_val = if bit_pos < entropy_bits {
                // Read from entropy bytes
                let byte_idx = bit_pos / 8;
                let bit_idx = 7 - (bit_pos % 8);
                (entropy[byte_idx] >> bit_idx) & 1
            } else {
                // Read from checksum
                let cs_bit_pos = bit_pos - entropy_bits;
                ((checksum >> (checksum_bits - 1 - cs_bit_pos)) & 1) as u8
            };
            if bit_val == 1 {
                idx |= 1 << (BITS_PER_WORD - 1 - b);
            }
        }
        indices.push(idx);
    }

    indices
}

/// Recover entropy from idiom indices (with checksum validation).
pub fn indices_to_entropy(indices: &[usize]) -> Result<Vec<u8>> {
    let word_count = indices.len();

    if !MNEMONIC_WORDS_VALID.contains(&word_count) {
        return Err(Error::InvalidLength(word_count));
    }

    // Validate index range
    for &idx in indices {
        if idx >= 4096 {
            return Err(Error::InvalidIndex(idx));
        }
    }

    indices_to_entropy_inner(indices)
}

/// Recover entropy from idiom indices (skips index range check; caller guarantees validity).
///
/// # Panics
///
/// Panics if `indices.len()` is not a valid word count.
pub(crate) fn indices_to_entropy_inner(indices: &[usize]) -> Result<Vec<u8>> {
    let word_count = indices.len();

    // Expand all indices into a bitstream
    let total_bits = word_count * BITS_PER_WORD;
    let mut bits = vec![0u8; (total_bits + 7) / 8];

    for (w, &idx) in indices.iter().enumerate() {
        for b in 0..BITS_PER_WORD {
            if (idx >> (BITS_PER_WORD - 1 - b)) & 1 == 1 {
                let bit_pos = w * BITS_PER_WORD + b;
                let byte_idx = bit_pos / 8;
                let bit_idx = 7 - (bit_pos % 8);
                bits[byte_idx] |= 1 << bit_idx;
            }
        }
    }

    // Compute entropy bits and checksum bits.
    // Uses table lookup instead of integer division to avoid silent rounding
    // errors if word counts are extended in the future.
    let entropy_bits = entropy_bits_for_word_count(word_count);
    let checksum_bits = total_bits - entropy_bits;
    let entropy_bytes = entropy_bits / 8;

    // Extract entropy
    let mut entropy = vec![0u8; entropy_bytes];
    entropy.copy_from_slice(&bits[..entropy_bytes]);

    // Extract and verify checksum
    let expected_checksum = calculate_checksum(&entropy, checksum_bits);

    let mut received_checksum = 0u32;
    for i in 0..checksum_bits {
        let bit_pos = entropy_bits + i;
        let byte_idx = bit_pos / 8;
        let bit_idx = 7 - (bit_pos % 8);
        if (bits[byte_idx] >> bit_idx) & 1 == 1 {
            received_checksum |= 1 << (checksum_bits - 1 - i);
        }
    }

    if expected_checksum != received_checksum {
        return Err(Error::ChecksumMismatch);
    }

    Ok(entropy)
}

/// Validate that an entropy byte length is valid (16/20/24/28/32 bytes).
pub fn validate_entropy_length(len: usize) -> bool {
    ENTROPY_BITS_VALID.contains(&(len * 8))
}

/// Validate that a mnemonic word count is valid (12/15/18/21/24 words).
pub fn validate_mnemonic_length(len: usize) -> bool {
    MNEMONIC_WORDS_VALID.contains(&len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_formula() {
        // Verify that checksum bits = entropy bits / 8
        for &bits in ENTROPY_BITS_VALID {
            assert_eq!(checksum_bits_for_entropy(bits), bits / 8);
        }
    }

    #[test]
    fn test_checksum() {
        let entropy = [0x00u8; 16];
        let checksum_bits = checksum_bits_for_entropy(128);
        let checksum = calculate_checksum(&entropy, checksum_bits);
        assert!(checksum < (1 << checksum_bits));
    }

    #[test]
    fn test_entropy_to_indices() {
        let entropy = [0x00u8; 16];
        let indices = entropy_to_indices(&entropy);
        assert_eq!(indices.len(), 12);
        // All indices should be within 0-4095
        for &idx in &indices {
            assert!(idx < 4096);
        }
    }

    #[test]
    fn test_indices_to_entropy_roundtrip() {
        let entropy = [0x00u8; 16];
        let indices = entropy_to_indices(&entropy);
        let recovered = indices_to_entropy(&indices).unwrap();
        assert_eq!(entropy, recovered.as_slice());
    }

    #[test]
    fn test_roundtrip_various_entropy() {
        for entropy_bits in ENTROPY_BITS_VALID {
            let entropy = vec![0xABu8; entropy_bits / 8];
            let indices = entropy_to_indices(&entropy);
            let recovered = indices_to_entropy(&indices).unwrap();
            assert_eq!(entropy, recovered);
        }
    }

    #[test]
    fn test_roundtrip_random_patterns() {
        // Test non-uniform entropy patterns (edge cases for bit manipulation)
        let test_entropies: Vec<Vec<u8>> = vec![
            vec![0xFF; 16],
            vec![0x55; 16],
            vec![0xAA; 16],
            vec![0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF],
            vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10],
        ];
        for entropy in &test_entropies {
            let indices = entropy_to_indices(entropy);
            let recovered = indices_to_entropy(&indices).unwrap();
            assert_eq!(entropy.as_slice(), recovered.as_slice());
        }
    }

    #[test]
    fn test_roundtrip_256bit() {
        let entropy = vec![0xDEu8; 32];
        let indices = entropy_to_indices(&entropy);
        assert_eq!(indices.len(), 24);
        let recovered = indices_to_entropy(&indices).unwrap();
        assert_eq!(entropy, recovered);
    }

    #[test]
    fn test_unchecked_roundtrip() {
        let entropy = [0x00u8; 16];
        let indices = entropy_to_indices(&entropy);
        let recovered = indices_to_entropy_inner(&indices).unwrap();
        assert_eq!(entropy, recovered.as_slice());
    }

    #[test]
    fn test_generate_entropy_zero_bits() {
        assert!(generate_entropy(0).is_err());
    }

    #[test]
    fn test_invalid_entropy_length() {
        assert!(generate_entropy(127).is_err());
        assert!(generate_entropy(257).is_err());
    }

    #[test]
    fn test_invalid_mnemonic_length() {
        let indices = vec![0; 13];
        assert!(indices_to_entropy(&indices).is_err());
    }

    #[test]
    fn test_invalid_index() {
        let indices = vec![4096; 12];
        assert!(indices_to_entropy(&indices).is_err());
    }

    #[test]
    fn test_validate_functions() {
        assert!(validate_entropy_length(16));
        assert!(validate_entropy_length(32));
        assert!(!validate_entropy_length(10));

        assert!(validate_mnemonic_length(12));
        assert!(validate_mnemonic_length(24));
        assert!(!validate_mnemonic_length(13));
    }
}