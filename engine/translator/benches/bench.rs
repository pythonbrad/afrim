#![allow(unused_imports, dead_code)]

#[cfg(feature = "rhai")]
use afrim_translator::Engine;
use afrim_translator::Translator;
#[cfg(not(target_arch = "wasm32"))]
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use indexmap::IndexMap;
use std::hint::black_box;

#[cfg(not(target_arch = "wasm32"))]
pub fn translate(c: &mut Criterion) {
    // Generates the dataset.
    let mut dictionary = IndexMap::new();
    let words = [
        "Aassiaiel",
        "Asiel",
        "AazAiel",
        "Aziel",
        "Aazaryas",
        "Uzziah",
        "Azaria",
        "Aazazya",
        "Azaziah",
        "Aazraiel",
        "Azriel",
        "Aebdaiel",
        "Abdeel",
    ];
    (0..100_000).for_each(|_| {
        words.into_iter().for_each(|word| {
            dictionary.insert(word.to_owned(), vec![word.to_owned()]);
        });
    });

    // Initializes the translator.
    let mut translator = Translator::new(dictionary, false, 0.7);

    // Registers a translator.
    #[cfg(feature = "rhai")]
    {
        let engine = Engine::new();
        let script_a = engine
            .compile("fn translate(input) { [input, \"\", input, false] }")
            .unwrap();
        let script_b = engine
            .compile("fn translate(input) { [input, \"\", input.len().to_string(), false] }")
            .unwrap();

        translator.register("give_back".to_owned(), script_a);
        translator.register("to_length".to_owned(), script_b);
    }

    // Generates candidates for testing.
    let mut candidates = Vec::new();
    "Aazraiel".chars().fold("".to_owned(), |word, character| {
        let new_word = format!("{word}{character}");
        candidates.push(new_word.clone());

        new_word
    });

    // Setup the benchmark.
    let mut group = c.benchmark_group("translate");
    for candidate in candidates.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(candidate),
            candidate,
            |b, candidate| {
                b.iter(|| {
                    translator.translate(black_box(candidate));
                });
            },
        );
    }
    group.finish();
}

#[cfg(not(target_arch = "wasm32"))]
criterion_group!(benches, translate);

#[cfg(not(target_arch = "wasm32"))]
criterion_main!(benches);

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("Can't run benchmarks on wasm32");
}
