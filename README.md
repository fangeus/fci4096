# fci4096

A BIP39-inspired mnemonic implementation using 4096 Chinese four-character idioms as the wordlist.

## Overview

fci4096 is a mnemonic system inspired by BIP39, replacing the English word list with 4096 standard four-character Chinese idioms. Each idiom encodes 12 bits of information (2^12 = 4096), providing higher information density compared to BIP39's 2048-word list (11 bits/word) for the same number of words.

### Differences from BIP39

| Feature | BIP39 | fci4096 |
|---------|-------|---------|
| Wordlist size | 2048 | 4096 |
| Bits per word | 11 | 12 |
| 12-word entropy | 128 bits | 128 bits |
| Checksum bits | 4 bits | 16 bits |
| Wordlist language | English | Chinese idioms |

## Encoding Scheme

With a 4096-word list, each idiom encodes 12 bits. The checksum is 1/8 of the entropy length:

| Word Count | Entropy | Checksum | Total Bits |
|------------|---------|----------|------------|
| 12 | 128 | 16 | 144 |
| 15 | 160 | 20 | 180 |
| 18 | 192 | 24 | 216 |
| 21 | 224 | 28 | 252 |
| 24 | 256 | 32 | 288 |

The checksum is the leading N bits of SHA-256(entropy).

## Installation

```toml
[dependencies]
fci4096 = "0.1"
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | Yes | Standard library support |
| `rand` | Yes | Random number generation (mnemonic generation) |
| `serde` | No | Serialize/Deserialize support |
| `wasm` | No | WASM bindings |

## Usage

### Generate a Mnemonic

```rust
use fci4096::{generate, IdiomMnemonicSize};

let mnemonic = generate(IdiomMnemonicSize::Idioms12)?;
println!("{}", mnemonic.phrase());
// e.g. 哀而不伤 按部就班 草木皆兵 ...
```

### From Entropy

```rust
use fci4096::from_entropy;

let entropy = [0x00u8; 16]; // 128 bits of entropy
let mnemonic = from_entropy(&entropy)?;
```

### Parse and Validate

```rust
use fci4096::{from_phrase, validate};

let phrase = "哀而不伤 按部就班 草木皆兵 ...";
if validate(phrase) {
    let mnemonic = from_phrase(phrase)?;
    let entropy = mnemonic.to_entropy()?;
}
```

### Generate a Seed

```rust
let seed = mnemonic.to_seed("passphrase");
// Returns [u8; 64], using PBKDF2-SHA512 with 4096 iterations
```

### CLI Examples

```bash
# Generate a 12-word mnemonic
cargo run --example generate 12

# Recover the seed
cargo run --example recover "idiom1 idiom2 idiom3 ..." [passphrase]
```

## Security

- **Entropy source**: Uses the `getrandom` crate for OS-level cryptographically secure random numbers
- **Checksum**: 128-bit entropy with 16-bit checksum (BIP39 uses only 4 bits), providing stronger error detection
- **Seed derivation**: PBKDF2-SHA512, 4096 iterations, passphrase is NFKD-normalized
- **Wordlist**: 4096 standard four-character idioms, sorted by pinyin, systematically reviewed to remove derogatory idioms

## License

MIT