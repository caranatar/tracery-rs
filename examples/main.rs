extern crate tracery;
use std::io::{self, Read};
use std::env;
use std::thread;
use std::time::Duration;

const DEFAULT: &'static str = r#"{
    "origin": [ "The #adjective# #color# #animal# jumps over the #adjective# #animal#" ],
    "adjective": [ "quick", "lazy", "slow", "tired", "drunk", "awake", "frantic" ],
    "color": [ "blue", "red", "yellow", "green", "purple", "orange", "pink", "brown", "black", "white" ],
    "animal": [ "dog", "fox", "cow", "horse", "chicken", "pig", "bird", "fish" ]
}"#;

fn main() {
    let args = env::args().skip(1).take(1);
    let mut src: String = args.collect();
    if src.len() == 0 {
        src = DEFAULT.to_string();
    }
    if src == "-" {
        let mut buffer = String::new();
        let _ = io::stdin().read_to_string(&mut buffer);
        src = buffer;
    }
    for _ in 0.. {
        println!("{}", tracery::flatten(&src).unwrap());
        thread::sleep(Duration::from_secs(2));
    }
}
