#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/idiom_data.rs"));

/// BIP39 Chinese idiom wordlist implementation.
///
/// Contains 4096 four-character idioms, each encoding 12 bits of information.
/// Uses a compile-time generated search array for O(log N) binary search lookup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChineseIdiomList;

impl ChineseIdiomList {
    /// Look up an idiom's index via binary search (O(log N)).
    ///
    /// Returns `None` immediately for non-four-character input (fast path).
    pub fn idiom_to_index(idiom: &str) -> Option<usize> {
        // Fast path: reject non-four-character input immediately
        if idiom.chars().count() != 4 {
            return None;
        }

        // IDIOM_SEARCH is sorted by Unicode codepoint, safe for binary search.
        // O(log 4096) ≈ 12 comparisons vs the original O(4096) linear scan.
        IDIOM_SEARCH
            .binary_search_by_key(&idiom, |&(word, _)| word)
            .ok()
            .map(|pos| IDIOM_SEARCH[pos].1)
    }

    /// Look up an idiom by its index (O(1) direct lookup).
    pub fn index_to_idiom(index: usize) -> Option<&'static str> {
        IDIOM_LIST.get(index).copied()
    }

    /// Get the pinyin for an idiom by index (only available when the CSV file is present).
    pub fn get_pinyin(index: usize) -> Option<&'static str> {
        IDIOM_WITH_PINYIN.get(index).map(|(_, pinyin)| *pinyin)
    }

    /// Return the wordlist size (fixed at 4096).
    pub fn len() -> usize {
        IDIOM_COUNT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idiom_list_length() {
        assert_eq!(ChineseIdiomList::len(), 4096);
    }

    #[test]
    fn test_index_to_idiom() {
        assert!(ChineseIdiomList::index_to_idiom(0).is_some());
        assert!(ChineseIdiomList::index_to_idiom(4095).is_some());
        assert_eq!(ChineseIdiomList::index_to_idiom(4096), None);
    }

    #[test]
    fn test_idiom_to_index() {
        let first_idiom = ChineseIdiomList::index_to_idiom(0).unwrap();
        assert_eq!(ChineseIdiomList::idiom_to_index(first_idiom), Some(0));
        assert_eq!(ChineseIdiomList::idiom_to_index("void"), None);
    }

    #[test]
    fn test_idiom_to_index_non_four_char() {
        // Empty string should return None immediately (fast path)
        assert_eq!(ChineseIdiomList::idiom_to_index(""), None);
        assert_eq!(ChineseIdiomList::idiom_to_index("abc"), None);
        assert_eq!(ChineseIdiomList::idiom_to_index("abcde"), None);
    }

    #[test]
    fn test_roundtrip() {
        for i in 0..100 {
            if let Some(idiom) = ChineseIdiomList::index_to_idiom(i) {
                assert_eq!(ChineseIdiomList::idiom_to_index(idiom), Some(i));
            }
        }
    }

    #[test]
    fn test_full_roundtrip() {
        // Verify index → idiom → index roundtrip for all 4096 entries
        for i in 0..4096 {
            let idiom = ChineseIdiomList::index_to_idiom(i).unwrap();
            assert_eq!(
                ChineseIdiomList::idiom_to_index(idiom),
                Some(i),
                "Roundtrip failed for index {}: idiom '{}'",
                i,
                idiom
            );
        }
    }

    #[test]
    fn test_no_duplicates() {
        // Verify wordlist has no duplicates
        let mut seen = std::collections::HashSet::new();
        for i in 0..4096 {
            let idiom = ChineseIdiomList::index_to_idiom(i).unwrap();
            assert!(
                seen.insert(idiom),
                "Duplicate idiom at index {}: '{}'",
                i,
                idiom
            );
        }
    }

    #[test]
    fn test_search_array_length() {
        assert_eq!(IDIOM_SEARCH.len(), 4096);
        // Verify the search array is sorted
        for i in 1..IDIOM_SEARCH.len() {
            assert!(
                IDIOM_SEARCH[i].0 >= IDIOM_SEARCH[i - 1].0,
                "IDIOM_SEARCH not sorted at index {}: '{}' < '{}'",
                i,
                IDIOM_SEARCH[i].0,
                IDIOM_SEARCH[i - 1].0
            );
        }
    }

    #[test]
    fn test_pinyin() {
        if IDIOM_WITH_PINYIN.is_empty() {
            // Skip pinyin test when CSV file is not available
            return;
        }
        let pinyin = ChineseIdiomList::get_pinyin(0);
        assert!(pinyin.is_some());
        assert!(!pinyin.unwrap().is_empty());
    }
}
