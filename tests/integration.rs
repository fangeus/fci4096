use fci4096::{
    from_entropy, from_phrase, generate, idiom_to_index, index_to_idiom, validate,
    IdiomMnemonicSize,
};

#[test]
fn test_generate_recover_validate() {
    let idiom_mnemonic = generate(IdiomMnemonicSize::Idioms12).unwrap();
    let phrase = idiom_mnemonic.phrase();

    assert!(validate(&phrase));

    let recovered = from_phrase(&phrase).unwrap();
    assert_eq!(idiom_mnemonic, recovered);

    let entropy = idiom_mnemonic.to_entropy().unwrap();
    let from_entropy_mnemonic = from_entropy(&entropy).unwrap();
    assert_eq!(idiom_mnemonic, from_entropy_mnemonic);
}

#[test]
fn test_all_idiom_mnemonic_sizes() {
    for size in [
        IdiomMnemonicSize::Idioms12,
        IdiomMnemonicSize::Idioms15,
        IdiomMnemonicSize::Idioms18,
        IdiomMnemonicSize::Idioms21,
        IdiomMnemonicSize::Idioms24,
    ]
    .iter()
    {
        let idiom_mnemonic = generate(*size).unwrap();
        assert_eq!(idiom_mnemonic.idioms().len(), size.idiom_count());

        let phrase = idiom_mnemonic.phrase();
        assert!(validate(&phrase));

        let recovered = from_phrase(&phrase).unwrap();
        assert_eq!(idiom_mnemonic, recovered);

        let entropy = idiom_mnemonic.to_entropy().unwrap();
        assert_eq!(entropy.len() * 8, size.entropy_bits());
    }
}

#[test]
fn test_seed_generation() {
    let idiom_mnemonic = generate(IdiomMnemonicSize::Idioms12).unwrap();
    let seed = idiom_mnemonic.to_seed("");
    assert_eq!(seed.len(), 64);

    let seed_with_passphrase = idiom_mnemonic.to_seed("test_passphrase");
    assert_ne!(seed, seed_with_passphrase);
}

#[test]
fn test_idiom_index_conversion() {
    let first_idiom = index_to_idiom(0).unwrap();
    let idx = idiom_to_index(first_idiom).unwrap();
    let recovered_idiom = index_to_idiom(idx).unwrap();
    assert_eq!(first_idiom, recovered_idiom);
}

#[test]
fn test_invalid_phrase() {
    assert!(!validate("这不是一个有效的助记词"));
    assert!(!validate("不存在的成语 另一个不存在的成语"));
}

#[test]
fn test_entropy_roundtrip() {
    let entropy = [
        0x12u8, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd,
        0xef,
    ];
    let idiom_mnemonic = from_entropy(&entropy).unwrap();
    let recovered_entropy = idiom_mnemonic.to_entropy().unwrap();
    assert_eq!(entropy, recovered_entropy.as_slice());
}

#[test]
fn test_empty_entropy() {
    assert!(from_entropy(&[]).is_err());
}

#[test]
fn test_invalid_entropy_length() {
    let invalid_entropy = [0x00u8; 17];
    assert!(from_entropy(&invalid_entropy).is_err());
}

#[test]
fn test_checksum_failure() {
    let idiom_mnemonic = generate(IdiomMnemonicSize::Idioms12).unwrap();
    let mut idioms: Vec<&str> = idiom_mnemonic.idioms();
    if let Some(alt_idiom) = index_to_idiom(1) {
        idioms[0] = alt_idiom;
    }
    let modified_phrase = idioms.join(" ");
    assert!(!validate(&modified_phrase));
}

#[test]
fn test_fullwidth_space_separator() {
    let entropy = [0x00u8; 16];
    let mnemonic = from_entropy(&entropy).unwrap();
    let phrase = mnemonic.phrase();
    let fullwidth = phrase.replace(' ', "\u{3000}"); // 全角空格
    let recovered = from_phrase(&fullwidth).unwrap();
    assert_eq!(mnemonic, recovered);
}

#[test]
fn test_tab_separator() {
    let entropy = [0x00u8; 16];
    let mnemonic = from_entropy(&entropy).unwrap();
    let phrase = mnemonic.phrase();
    let tabbed = phrase.replace(' ', "\t");
    let recovered = from_phrase(&tabbed).unwrap();
    assert_eq!(mnemonic, recovered);
}

#[test]
fn test_mixed_separator() {
    let entropy = [0x00u8; 16];
    let mnemonic = from_entropy(&entropy).unwrap();
    let phrase = mnemonic.phrase();
    // 混合空格、全角空格和 tab
    let parts: Vec<&str> = phrase.split(' ').collect();
    let mixed = format!(
        "{}  {}　{}\t{}  {}　{}\t{}  {}　{}\t{}  {}　{}",
        parts[0],
        parts[1],
        parts[2],
        parts[3],
        parts[4],
        parts[5],
        parts[6],
        parts[7],
        parts[8],
        parts[9],
        parts[10],
        parts[11]
    );
    let recovered = from_phrase(&mixed).unwrap();
    assert_eq!(mnemonic, recovered);
}

#[test]
fn test_boundary_index_zero() {
    assert!(index_to_idiom(0).is_some());
    let idiom = index_to_idiom(0).unwrap();
    assert_eq!(idiom_to_index(idiom), Some(0));
}

#[test]
fn test_boundary_index_max() {
    assert!(index_to_idiom(4095).is_some());
    let idiom = index_to_idiom(4095).unwrap();
    assert_eq!(idiom_to_index(idiom), Some(4095));
}

#[test]
fn test_index_out_of_bounds() {
    assert!(index_to_idiom(4096).is_none());
    assert!(index_to_idiom(usize::MAX).is_none());
}

#[test]
fn test_idiom_to_index_invalid() {
    assert_eq!(idiom_to_index(""), None);
    assert_eq!(idiom_to_index("三字"), None);
    assert_eq!(idiom_to_index("五个字的成语"), None);
    assert_eq!(idiom_to_index("abcd"), None);
}

#[test]
fn test_validate_edge_cases() {
    assert!(!validate(""));
    assert!(!validate("   "));
    assert!(!validate("单个成语"));
    assert!(!validate("哀而不伤")); // 只有一个成语
}

#[test]
fn test_seed_iterations_configurable() {
    use fci4096::mnemonic::IdiomMnemonic;
    let entropy = [0x00u8; 16];
    let mnemonic = IdiomMnemonic::from_entropy(&entropy).unwrap();

    let seed_default = mnemonic.to_seed("");
    let seed_custom = mnemonic.to_seed_with_iterations("", 2048);

    assert_ne!(seed_default, seed_custom);
}

#[test]
fn test_seed_deterministic() {
    let entropy = [0xabu8; 16];
    let mnemonic1 = from_entropy(&entropy).unwrap();
    let mnemonic2 = from_entropy(&entropy).unwrap();
    assert_eq!(mnemonic1.to_seed("test"), mnemonic2.to_seed("test"));
}

#[test]
fn test_phrase_is_deterministic() {
    let entropy = [0x42u8; 16];
    let mnemonic1 = from_entropy(&entropy).unwrap();
    let mnemonic2 = from_entropy(&entropy).unwrap();
    assert_eq!(mnemonic1.phrase(), mnemonic2.phrase());
}
