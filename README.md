# Tracery
A Text-Expansion Library for Rust

[![Crates.io](https://img.shields.io/crates/v/tracery.svg)](https://crates.io/crates/tracery)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Documentation](https://docs.rs/tracery/badge.svg)](https://docs.rs/tracery)
[![Coverage Status](https://coveralls.io/repos/github/caranatar/tracery-rs/badge.svg?branch=master)](https://coveralls.io/github/caranatar/tracery-rs?branch=master)

Tracery was originally a javascript library written by [galaxykate](https://github.com/galaxykate), and is available at <https://github.com/galaxykate/tracery>.
It accepts a set of rules, and produces a single string according to specific syntax in the rule set.

If a string in the rule set contains a word surrounded by `#` symbols, it will be used to select a piece of text from the rule named by the word between the `#` symbols.

Here are some examples:

```rust
let source = r#"{
    "origin": [ "The #adjective# #color# #animal# jumps over the #adjective# #animal#" ],
    "adjective": [ "quick", "lazy", "slow", "tired", "drunk", "awake", "frantic" ],
    "color": [ "blue", "red", "yellow", "green", "purple", "orange", "pink", "brown", "black", "white" ],
    "animal": [ "dog", "fox", "cow", "horse", "chicken", "pig", "bird", "fish" ]
}"#;

for _ in 0..3 {
    println!("{}", tracery::flatten(source));
}

// The quick brown fox jumps over the lazy dog
// The slow red chicken jumps over the orange bird
// The drunk purple bird jumps over the green horse
```

(You can run this exact example by running `cargo run --example readme`. If you want to try out some of your own, try piping a JSON grammar to it like this: `echo '<json here>' | cargo run --example main -- -`)
