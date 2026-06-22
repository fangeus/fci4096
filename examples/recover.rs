//! Example: Recover entropy and seed from an idiom mnemonic phrase.
//!
//! Usage:
//! ```bash
//! cargo run --example recover "<mnemonic phrase>" [passphrase]
//! ```

use fci4096::{from_phrase, validate};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: recover <mnemonic phrase> [passphrase]");
        return;
    }
    
    let phrase = &args[1];
    let passphrase = if args.len() > 2 { &args[2] } else { "" };
    
    if !validate(phrase) {
        println!("Error: invalid mnemonic");
        return;
    }
    
    let idiom_mnemonic = from_phrase(phrase).unwrap();
    
    let seed = idiom_mnemonic.to_seed(passphrase);
    println!("Seed (64 bytes):");
    println!("{}", hex::encode(&seed));
    
    let entropy = idiom_mnemonic.to_entropy().unwrap();
    println!("\nEntropy ({} bits):", entropy.len() * 8);
    println!("{}", hex::encode(&entropy));
    
    println!("\nMnemonic validation: valid ✓");
}