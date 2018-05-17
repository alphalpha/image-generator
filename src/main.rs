extern crate image_generator;

use image_generator::Config;
use std::env;
use std::process;

fn help() -> String {
    String::from("usage: \
    image-generator [<images_path>] [<Camera_ID>] [<font_path] [<x-coordinate> <y-coordinate> <width> <height>] [text_before_date_and_time]\n\
    example: image-generator src/test_data MC105 src/DejaVuSans.ttf 100 100 100 100")
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
