extern crate image;
extern crate imageproc;
extern crate rusttype;

use std::path::{Path, PathBuf};
use std::env;
use std::{fs, io};
use imageproc::drawing::draw_text_mut;
use image::{DynamicImage, GenericImage, Rgb, RgbImage};
use rusttype::{FontCollection, Scale};


fn mean_color(img : &DynamicImage) -> Result<Rgb<u8>, io::Error> {
    let dimensions = img.dimensions();
    let num_pixels = dimensions.0 * dimensions.1;

    let color : Vec<u8> = img.pixels()
        .fold(vec![0u32, 0u32, 0u32], |mut acc, pixel| {
            for i in 0..acc.len() { acc[i] += pixel.2[i] as u32; }
            acc
        })
        .iter()
        .map(|c| { (c / num_pixels) as u8 })
        .collect();

    println!("{:?}", color);
    Ok(Rgb([color[0], color[1], color[2]]))
}

fn image_paths(dir : &str) -> Result<Vec<PathBuf>, io::Error> {
    let paths : Vec<_> = fs::read_dir(dir)?
        .map(|f| f.unwrap().path())
        .filter(|f| f.extension().is_some())
        .collect();
    Ok(paths)
}

fn main() {
    if env::args().count() != 3 {
        panic!("Please enter a target file path")
    };
    let args: Vec<String> = env::args().collect();
    let metadata = Path::new(&args[1]).metadata().expect("Getting Metadata failed");
    if !metadata.is_dir() {
        panic!("Must be a directory");
    }

    let font = Vec::from(include_bytes!("DejaVuSans.ttf") as &[u8]);
    let font = FontCollection::from_bytes(font).into_font().expect("Could not load font");
    let height = 22.4;
    let scale = Scale { x: height * 1.0, y: height };

    let out_path = Path::new(&args[2]);
    let in_paths = image_paths(&args[1]).expect("Could not read files in directory");

    for in_path in in_paths.iter() {
        let file_name = out_path.to_str().unwrap().to_string()
            + "/"
            + in_path.file_stem().unwrap().to_str().unwrap()
            + "_green."
            + in_path.extension().unwrap().to_str().unwrap();
        let in_image = image::open(&in_path).expect("Opening image failed");

        let color = mean_color(&in_image).expect("Could not calculate mean color");
        let mut image = RgbImage::new(in_image.dimensions().0, in_image.dimensions().1);
        for p in image.pixels_mut() { *p = color; }

        let text = in_path.file_stem().unwrap().to_str().unwrap();
        draw_text_mut(&mut image, Rgb([255u8, 255u8, 255u8]), 10, 10, scale, &font, text);

        let _ = image.save(file_name).expect("Could not save file");
    }
}

