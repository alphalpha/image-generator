extern crate image;
extern crate imageproc;
extern crate rusttype;

use std::path::{Path, PathBuf};
use std::env;
use std::{fs, io};
use std::io::{Error, ErrorKind};
use imageproc::drawing::draw_text_mut;
use imageproc::rect::Rect;
use image::{DynamicImage, GenericImage, Rgb, RgbImage};
use rusttype::{FontCollection, Scale};

fn mean_color(img: &DynamicImage, rect: &Rect) -> Result<Rgb<u8>, io::Error> {
    let num_pixels = rect.width() * rect.height();

    let color: Vec<u8> = img.pixels()
        .filter(|f| {
            (f.0 >= rect.left() as u32 && f.0 < (rect.left() as u32 + rect.width() as u32))
                && (f.1 >= rect.top() as u32 && f.1 < (rect.top() as u32 + rect.height() as u32))
        })
        .fold(vec![0u32, 0u32, 0u32], |mut acc, pixel| {
            for i in 0..acc.len() {
                acc[i] += pixel.2[i] as u32;
            }
            acc
        })
        .iter()
        .map(|c| (c / num_pixels) as u8)
        .collect();

    println!("{:?}", color);
    Ok(Rgb([color[0], color[1], color[2]]))
}

fn image_paths(dir: &str) -> Result<Vec<PathBuf>, io::Error> {
    let paths: Vec<_> = fs::read_dir(dir)?
        .map(|f| f.unwrap().path())
        .filter(|f| f.extension().is_some())
        .collect();
    Ok(paths)
}

fn citing(name: &str) -> Result<String, std::io::Error> {
    let mut parts: Vec<&str> = name.splitn(5, '_').collect();
    match parts.len() {
        5 => {
            let parts = parts.split_off(2);
            let (year, rest) = parts[1].split_at(4);
            let (month, day) = rest.split_at(2);
            let date = day.to_string() + "." + month + "." + year;
            let (hour, rest) = parts[2].split_at(2);
            let (minutes, seconds) = rest.split_at(2);
            let time = hour.to_string() + ":" + minutes + ":" + seconds;
            Ok(String::from(
                parts[0].to_string() + ", " + &date + ", " + &time,
            ))
        }
        _ => Err(Error::new(
            ErrorKind::Other,
            String::from("File: \"".to_string() + name + "\" has wrong name format"),
        )),
    }
}

fn output_file_path(target_dir: &Path, source_file: &Path) -> Result<PathBuf, Error> {
    let mut stem = source_file.file_stem().unwrap().to_os_string();
    stem.push("_green");
    Ok(target_dir
        .join(stem)
        .with_extension(source_file.extension().unwrap()))
    //None => Err(Error::new(ErrorKind::Other, "Problem"),
}

fn obtain_area(args: Vec<String>) -> Result<Rect, Error> {
    if args.len() != 4 {
        return Err(Error::new(
            ErrorKind::Other,
            String::from("Not enough arguments to define the area to be analzed"),
        ));
    }

    let rect: Vec<i32> = args.into_iter()
        .map(|n| n.parse().expect("Cannot convert to integer!"))
        .collect();
    Ok(Rect::at(rect[0], rect[1]).of_size(rect[2] as u32, rect[3] as u32))
}

fn main() {
    if env::args().count() != 7 {
        panic!("Please enter a target file path")
    };
    let args: Vec<String> = env::args().collect();
    let metadata = Path::new(&args[1])
        .metadata()
        .expect("Getting Metadata failed");
    if !metadata.is_dir() {
        panic!("Must be a directory");
    }

    let rect = match obtain_area(args.clone().split_off(3)) {
        Ok(r) => r,
        Err(e) => panic!("{}", e),
    };

    let font = Vec::from(include_bytes!("DejaVuSans.ttf") as &[u8]);
    let font = FontCollection::from_bytes(font)
        .into_font()
        .expect("Could not load font");
    let height = 22.4;
    let scale = Scale {
        x: height * 1.0,
        y: height,
    };

    let out_path = Path::new(&args[2]);
    let in_paths = image_paths(&args[1]).expect("Could not read files in directory");

    for in_path in in_paths.iter() {
        let in_image = image::open(&in_path).expect("Opening image failed");
        let color = mean_color(&in_image, &rect).expect("Could not calculate mean color");

        let mut image = RgbImage::new(in_image.dimensions().0, in_image.dimensions().1);
        for p in image.pixels_mut() {
            *p = color;
        }

        let text = match citing(in_path.file_stem().unwrap().to_str().unwrap()) {
            Ok(c) => c,
            Err(e) => panic!("{}", e),
        };

        draw_text_mut(
            &mut image,
            Rgb([255u8, 255u8, 255u8]),
            10,
            10,
            scale,
            &font,
            text.as_str(),
        );

        let path = match output_file_path(&out_path, &in_path) {
            Ok(p) => p,
            Err(e) => panic!("{}", e),
        };
        let _ = image.save(path).expect("Could not save file");
    }
}
