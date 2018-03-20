extern crate image_generator;

use std::env;
use std::process;
use image_generator::Config;

fn help() -> String {
    String::from("usage: \
    image-generator [<images_path>] [<font_path] [<x-coordinate> <y-coordinate> <width> <height>]\n\
    example: image-generator src/test_data src/DejaVuSans.ttf 100 100 100 100")
}

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|e| {
        eprintln!("Problem parsing arguments. {}\n\n{}", e, help());
        process::exit(1);
    });

    if let Err(e) = image_generator::run(config) {
        eprintln!("Application Error: {}", e);
        process::exit(1);
    }
}
