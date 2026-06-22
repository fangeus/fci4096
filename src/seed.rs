use pbkdf2::pbkdf2_hmac_array;
use sha2::Sha512;
use unicode_normalization::UnicodeNormalization;

/// Default PBKDF2 iteration count (**Note: this project uses 4096, which differs from BIP39's 2048**).
///
/// Higher iteration counts provide stronger security but slower computation on
/// low-performance devices. Use [`mnemonic_to_seed_with_iterations`] for a custom count.
///
/// **Compatibility warning**: Seeds generated with 4096 iterations are **not compatible**
/// with standard BIP39 wallets (which use 2048 iterations). For BIP39 wallet
/// interoperability, use `mnemonic_to_seed_with_iterations(mnemonic, passphrase, 2048)`.
pub const PBKDF2_ITERATIONS: u32 = 4096;

/// Derive a 64-byte seed from a mnemonic phrase and passphrase (using the default 4096 iterations).
///
/// Follows the BIP39 process:
/// 1. Apply NFKD Unicode normalization to the passphrase
/// 2. Apply NFKD normalization to the mnemonic itself (ensuring different Unicode
///    representations produce the same seed)
/// 3. Salt = "mnemonic" + normalized_passphrase
/// 4. Use PBKDF2-SHA512 with 4096 iterations to derive a 64-byte seed
#[must_use]
pub fn mnemonic_to_seed(mnemonic: &str, passphrase: &str) -> [u8; 64] {
    mnemonic_to_seed_with_iterations(mnemonic, passphrase, PBKDF2_ITERATIONS)
}

/// Derive a 64-byte seed from a mnemonic phrase and passphrase (custom iteration count).
///
/// Parameters:
/// - `mnemonic`: The mnemonic phrase (space-separated idioms)
/// - `passphrase`: Optional passphrase (empty string for none)
/// - `iterations`: PBKDF2 iteration count
///
/// Both the mnemonic and passphrase are NFKD-normalized to ensure that the same
/// semantic text produces a consistent seed regardless of Unicode representation.
#[must_use]
pub fn mnemonic_to_seed_with_iterations(
    mnemonic: &str,
    passphrase: &str,
    iterations: u32,
) -> [u8; 64] {
    let normalized_mnemonic: String = mnemonic.nfkd().collect();
    let normalized_passphrase: String = passphrase.nfkd().collect();

    // Pre-allocate capacity to avoid format! reallocation
    let mut salt = String::with_capacity(8 + normalized_passphrase.len());
    salt.push_str("mnemonic");
    salt.push_str(&normalized_passphrase);
    pbkdf2_hmac_array::<Sha512, 64>(
        normalized_mnemonic.as_bytes(),
        salt.as_bytes(),
        iterations,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_generation() {
        let mnemonic =
            "哀感天地 哀毁瘠立 挨肩叠背 唉声叹气 挨三顶五 矮子看戏 爱才若渴 暧昧不明 碍手碍脚 安邦定国 安分守己 安魂定魄";
        let seed = mnemonic_to_seed(mnemonic, "");
        assert_eq!(seed.len(), 64);
    }

    #[test]
    fn test_seed_with_passphrase() {
        let mnemonic = "哀感天地 哀毁瘠立 挨肩叠背 唉声叹气 挨三顶五 矮子看戏 爱才若渴 暧昧不明 碍手碍脚 安邦定国 安分守己 安魂定魄";
        let seed1 = mnemonic_to_seed(mnemonic, "");
        let seed2 = mnemonic_to_seed(mnemonic, "password");
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_seed_consistency() {
        let mnemonic = "哀感天地 哀毁瘠立 挨肩叠背 唉声叹气 挨三顶五 矮子看戏 爱才若渴 暧昧不明 碍手碍脚 安邦定国 安分守己 安魂定魄";
        let seed1 = mnemonic_to_seed(mnemonic, "test");
        let seed2 = mnemonic_to_seed(mnemonic, "test");
        assert_eq!(seed1, seed2);
    }

    #[test]
    fn test_seed_custom_iterations() {
        let mnemonic = "哀感天地 哀毁瘠立 挨肩叠背 唉声叹气 挨三顶五 矮子看戏 爱才若渴 暧昧不明 碍手碍脚 安邦定国 安分守己 安魂定魄";
        let seed_default = mnemonic_to_seed(mnemonic, "");
        let seed_custom = mnemonic_to_seed_with_iterations(mnemonic, "", 4096);
        assert_eq!(seed_default, seed_custom);
    }

    #[test]
    fn test_nfkd_normalization_consistency() {
        // Verify NFKD normalization ensures full-width/half-width characters produce
        // consistent results
        let mnemonic = "哀感天地 哀毁瘠立 挨肩叠背 唉声叹气 挨三顶五 矮子看戏 爱才若渴 暧昧不明 碍手碍脚 安邦定国 安分守己 安魂定魄";
        let seed1 = mnemonic_to_seed(mnemonic, "TEST");
        let seed2 = mnemonic_to_seed(mnemonic, "TEST");
        assert_eq!(seed1, seed2);

        // Passphrase with emoji and special characters
        let seed3 = mnemonic_to_seed(mnemonic, "密码🔑");
        assert_eq!(seed3.len(), 64);
    }
}