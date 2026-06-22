use wasm_bindgen::prelude::*;
use crate::{ChineseIdiomList, IdiomMnemonic, IdiomMnemonicSize};

/// Generate a random mnemonic (WASM export).
///
/// `idiom_count` must be 12, 15, 18, 21, or 24.
/// Returns a space-separated idiom phrase string.
///
/// Note: Requires the `rand` feature. On wasm32 targets, the `getrandom`
/// crate's `js` feature is also needed (auto-configured in `Cargo.toml`).
#[wasm_bindgen]
#[cfg(feature = "rand")]
pub fn generate(idiom_count: u32) -> Result<String, JsValue> {
    let size = match idiom_count {
        12 => IdiomMnemonicSize::Idioms12,
        15 => IdiomMnemonicSize::Idioms15,
        18 => IdiomMnemonicSize::Idioms18,
        21 => IdiomMnemonicSize::Idioms21,
        24 => IdiomMnemonicSize::Idioms24,
        _ => {
            return Err(JsValue::from_str(
                "Invalid idiom count. Must be 12, 15, 18, 21, or 24",
            ))
        }
    };

    let idiom_mnemonic =
        IdiomMnemonic::generate(size).map_err(|e| JsValue::from_str(&format!("{}", e)))?;
    Ok(idiom_mnemonic.phrase())
}

/// Convert mnemonic to seed (WASM export).
///
/// `phrase` is a space-separated idiom string.
/// `passphrase` is an optional password (`None` means empty password).
/// Returns a 64-byte seed.
#[wasm_bindgen]
pub fn to_seed(phrase: &str, passphrase: Option<&str>) -> Result<Vec<u8>, JsValue> {
    let idiom_mnemonic =
        IdiomMnemonic::from_phrase(phrase).map_err(|e| JsValue::from_str(&format!("{}", e)))?;
    let pass = passphrase.unwrap_or("");
    let seed = idiom_mnemonic.to_seed(pass);
    Ok(seed.to_vec())
}

/// Validate a mnemonic phrase (WASM export).
///
/// Returns `true` if the checksum passes and the phrase is valid.
#[wasm_bindgen]
pub fn validate(phrase: &str) -> bool {
    IdiomMnemonic::validate(phrase)
}

/// Look up an idiom's index (WASM export).
///
/// Returns the index (0..4095) of the idiom in the wordlist, or `None` if not found.
#[wasm_bindgen]
pub fn idiom_to_index(idiom: &str) -> Option<usize> {
    ChineseIdiomList::idiom_to_index(idiom)
}

/// Look up an idiom by its index (WASM export).
///
/// Returns the idiom string, or `None` if the index is out of bounds.
#[wasm_bindgen]
pub fn index_to_idiom(idx: usize) -> Option<&'static str> {
    ChineseIdiomList::index_to_idiom(idx)
}

/// Get pinyin for an idiom (WASM export).
///
/// Returns the pinyin string for the idiom at the given index.
#[wasm_bindgen]
pub fn get_pinyin(idx: usize) -> Option<&'static str> {
    ChineseIdiomList::get_pinyin(idx)
}

/// Get the total idiom count (WASM export).
#[wasm_bindgen]
pub fn idiom_count() -> usize {
    ChineseIdiomList::len()
}