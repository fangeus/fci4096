# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-16

### Added

- BIP39-inspired mnemonic implementation using 4096 Chinese four-character idioms
- Entropy generation (128/160/192/224/256 bits) from OS random source
- Checksum calculation (1/8 of entropy length)
- Conversion between entropy bytes and idiom indices
- Idiom wordlist with O(log N) binary search via Unicode-codepoint-sorted search array
- Pinyin lookup support (when CSV data is available)
- `IdiomMnemonic` struct with:
  - `from_entropy()` / `from_phrase()` / `generate()` constructors
  - `to_entropy()` / `phrase()` / `idioms()` / `validate()` methods
  - `to_seed()` with PBKDF2-SHA512 (default 4096 iterations)
- `IdiomMnemonicSize` enum (12/15/18/21/24 idioms)
- WASM bindings with `generate()` / `from_phrase()` / `validate()` / `phrase()` / `to_seed()`
- Internal `Vec<u16>` index storage for reduced memory usage
- Feature flags: `std`, `rand`, `wasm`, `serde`
- Comprehensive unit tests (35 tests) and integration tests (20 tests)
- Example programs: generate, recover

[0.1.0]: https://github.com/fangeus/fci4096/releases/tag/v0.1.0