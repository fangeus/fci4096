# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.4] - 2026-06-23

### Fixed

- GitHub Actions release workflow: add `permissions: contents: write` to the
  `github-release` job to fix "Resource not accessible by integration" error
- Upgrade `actions/checkout` from v4 to v5 (Node.js 24 runtime)
- Upgrade `softprops/action-gh-release` from v2 to v3 (Node.js 24 runtime)

## [0.1.3] - 2026-06-23

### Fixed

- Restore English CSV header (`index,chinese_idiom,pinyin`) after data regeneration
- Restore JSON keys (`chinese_idioms`/`chinese_idiom`) after data regeneration
- Remove trailing newline from `four_chinese_idiom.txt` after data regeneration
- Update `tools/generate_idiom_data.py` to produce correct formats by default

### Changed

- Updated idiom wordlist data (csv, json, txt, zip.json)

## [0.1.2] - 2026-06-23

### Fixed

- Fix indentation in `build.rs` to satisfy `cargo fmt --check`

## [0.1.1] - 2026-06-23

### Changed

- Standardize CSV header to English: `index,chinese_idiom,pinyin`
- Rename JSON fields: `words` → `chinese_idioms`, `word` → `chinese_idiom`
- Remove trailing newline from `four_chinese_idiom.txt`
- Update `build.rs` header validation to match the new CSV header
- Update source files, examples, tests, and benchmarks

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

[0.1.4]: https://github.com/fangeus/fci4096/releases/tag/v0.1.4
[0.1.3]: https://github.com/fangeus/fci4096/releases/tag/v0.1.3
[0.1.2]: https://github.com/fangeus/fci4096/releases/tag/v0.1.2
[0.1.1]: https://github.com/fangeus/fci4096/releases/tag/v0.1.1
[0.1.0]: https://github.com/fangeus/fci4096/releases/tag/v0.1.0