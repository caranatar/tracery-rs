# tracery
Rust implementation of the tracery generative grammar language.

[![Crates.io](https://img.shields.io/crates/v/tracery.svg)](https://crates.io/crates/tracery)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Documentation](https://docs.rs/tracery/badge.svg)](https://docs.rs/tracery)
[![Coverage Status](https://coveralls.io/repos/github/caranatar/tracery-rs/badge.svg?branch=master)](https://coveralls.io/github/caranatar/tracery-rs?branch=master)

This library is a Rust port/implementation of [tracery], the generative
grammar language designed and created by [Kate Compton]. Given a set of
rules written in the tracery syntax, it will use them to procedurally
generate strings of text. For more information about the tracery language,
see [Language Concepts].

## Usage
Usage of the library can be divided into two areas: creation of grammars and
the generation of output strings.

### Grammar Creation
Grammars can be created using the [`grammar!`] macro, from an any iterable
rust object of strings and associated lists of strings, or for compatibility
with the original tracery, from a string representing a JSON map.

#### The grammar! macro
Accepts input in the form `"key" => [ "list", "of", "rules" ]` or, in the
case of a key having only one rule, `"key" => "rule"`. Equivalent to
manually building a map and then calling [`Grammar::from_map`]

```rust
use tracery::grammar;
let g = grammar! {
    "origin" => "#tool# is #description#!",
    "tool" => "tracery",
    "description" => [ "fun", "awesome" ]
}?;
```

#### From a map/iterator
A grammar can be created from any object implementing, essentially,
`IntoIterator<Item = (Into<String>, Into<Vec<Into<String>>)>`. For example,
`HashMap<String, Vec<String>>` or `BTreeMap<&str, &[&str]>`.

```rust
let map = hashmap! {
    "origin" => vec![ "#tool# is #description#!" ],
    "tool" => vec![ "tracery" ],
    "description" => vec![ "fun", "awesome" ]
};
let g = tracery::from_map(map)?;
```

#### From a JSON string
For compatibility with the original tracery, a Grammar can be created from a
string representing a JSON object. This feature is controlled by the
`tracery_json` feature, which is enabled by default. It can be turned off if
you do not require this functionality.

```rust
let json = r##"{
    "origin": [ "#tool# is #description#!" ],
    "tool": [ "tracery" ],
    "description": [ "fun", "awesome" ]
}"##;
let g = tracery::from_json(json)?;
```

## Generating output strings
There are two methods for getting a generated output string from a created
Grammar: [`execute`] and [`flatten`]. Generally, [`execute`] should be
preferred if possible.

### execute
[`execute`] takes two parameters: the rule to expand and an RNG to use
during generation. The RNG can be any type implementing [`rand::Rng`].

```rust
use tracery::grammar;
let mut g = grammar! {
    "origin" => "#tool# is #description#!",
    "tool" => "tracery",
    "description" => [ "fun", "awesome" ]
}?;
// Generate an output (either "tracery is fun!" or "tracery is awesome!")
let key = String::from("origin");
let output = g.execute(&key, &mut rand::thread_rng())?;
```

[`execute`] generates its output using the Grammar in-place. Since Grammars
are allowed to modify their own rule stacks, [`execute`] must take a `&mut
self` reference. This means that any modifications made during an execution
will persist in the Grammar.

```rust
use tracery::grammar;
// This time, origin has a side-effect: it creates the rule 'aside'
let mut g = grammar! {
    "origin" => "#[aside:Rust is, too]tool# is #description#!",
    "tool" => "tracery",
    "description" => [ "fun", "awesome" ]
}?;
// Generate an output (either "tracery is fun!" or "tracery is awesome!")
let key = String::from("origin");
let output = g.execute(&key, &mut rand::thread_rng())?;
// The previous call to execute created the 'aside' rule
let key = String::from("aside");
// Generates the string "Rust is, too"
let output = g.execute(&key, &mut rand::thread_rng())?;
```

### flatten
[`flatten`], unlike [`execute`], always operates on the default rule of the
Grammar ("origin" by default), but like [`execute`], takes an instance of
[`rand::Rng`] to use during generation. In addition, [`flatten`] creates a
clone of the Grammar to use during generation, then discards it, which means
that any side-effects that occur will be discarded when it's done.

```rust
use tracery::grammar;
let g = grammar! {
    "origin" => "#tool# is #description#!",
    "tool" => "tracery",
    "description" => [ "fun", "awesome" ]
}?;
// Generate an output (either "tracery is fun!" or "tracery is awesome!")
let output = g.flatten(&mut rand::thread_rng())?;
```

[tracery]: https://tracery.io/
[Kate Compton]: http://www.galaxykate.com/
[Language Concepts]: https://docs.rs/tracery/latest/tracery/index.html#language-concepts
[`grammar!`]: https://docs.rs/tracery/latest/tracery/macro.grammar.html
[`Grammar::from_map`]: https://docs.rs/tracery/latest/tracery/struct.Grammar.html#method.from_map
[`execute`]: https://docs.rs/tracery/latest/tracery/struct.Grammar.html#method.execute
[`flatten`]: https://docs.rs/tracery/latest/tracery/struct.Grammar.html#method.flatten
[`rand::Rng`]: http://docs.rs/rand/latest/rand/trait.Rng.html

License: MIT/Apache-2.0
