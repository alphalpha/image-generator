extern crate image_generator;

use std::env;
use std::process;
use image_generator::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|e| {
        eprintln!("Problem parsing arguments: {}", e);
        process::exit(1);
    });

    if let Err(e) = image_generator::run(config) {
        eprintln!("Application Error: {}", e);
        process::exit(1);
    }
}
