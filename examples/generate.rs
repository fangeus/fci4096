//! Example: Generate a random idiom mnemonic.
//!
//! Usage:
//! ```bash
//! cargo run --example generate [idiom_count]
//! ```
//! `idiom_count` must be 12, 15, 18, 21, or 24 (defaults to 12).

use fci4096::{generate, IdiomMnemonicSize};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let size = if args.len() > 1 {
        match args[1].as_str() {
            "12" => IdiomMnemonicSize::Idioms12,
            "15" => IdiomMnemonicSize::Idioms15,
            "18" => IdiomMnemonicSize::Idioms18,
            "21" => IdiomMnemonicSize::Idioms21,
            "24" => IdiomMnemonicSize::Idioms24,
            _ => {
                println!("Invalid idiom count. Using default 12.");
                IdiomMnemonicSize::Idioms12
            }
        }
    } else {
        IdiomMnemonicSize::Idioms12
    };
    
    let idiom_mnemonic = generate(size).unwrap();
    println!("Generated mnemonic ({} idioms):", size.idiom_count());
    println!("{}", idiom_mnemonic.phrase());
    
    let entropy = idiom_mnemonic.to_entropy().unwrap();
    println!("\nEntropy ({} bits):", entropy.len() * 8);
    println!("{}", hex::encode(&entropy));
}