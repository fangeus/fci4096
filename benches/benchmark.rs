use criterion::{criterion_group, criterion_main, Criterion};
use fci4096::{from_entropy, from_phrase, generate, IdiomMnemonic, IdiomMnemonicSize};

fn bench_generate_12(c: &mut Criterion) {
    c.bench_function("generate_12_idioms", |b| {
        b.iter(|| generate(IdiomMnemonicSize::Idioms12).unwrap())
    });
}

fn bench_generate_24(c: &mut Criterion) {
    c.bench_function("generate_24_idioms", |b| {
        b.iter(|| generate(IdiomMnemonicSize::Idioms24).unwrap())
    });
}

fn bench_idiom_mnemonic_to_seed(c: &mut Criterion) {
    let idiom_mnemonic = generate(IdiomMnemonicSize::Idioms12).unwrap();
    c.bench_function("idiom_mnemonic_to_seed", |b| {
        b.iter(|| idiom_mnemonic.to_seed(""))
    });
}

fn bench_from_entropy(c: &mut Criterion) {
    let entropy = [0x00u8; 16];
    c.bench_function("from_entropy_128bit", |b| {
        b.iter(|| from_entropy(&entropy).unwrap())
    });
}

fn bench_from_phrase(c: &mut Criterion) {
    let idiom_mnemonic = generate(IdiomMnemonicSize::Idioms12).unwrap();
    let phrase = idiom_mnemonic.phrase();
    c.bench_function("from_phrase_12_idioms", |b| {
        b.iter(|| from_phrase(&phrase).unwrap())
    });
}

criterion_group!(
    benches,
    bench_generate_12,
    bench_generate_24,
    bench_idiom_mnemonic_to_seed,
    bench_from_entropy,
    bench_from_phrase
);

criterion_main!(benches);
