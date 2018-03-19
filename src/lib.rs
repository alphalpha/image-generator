extern crate image;
extern crate imageproc;
extern crate rusttype;

use std::path::{Path, PathBuf};
use std::{fs, io};
use std::error::Error;
use std::io::{ErrorKind, Read};
use imageproc::drawing::draw_text_mut;
use imageproc::rect::Rect;
use image::{DynamicImage, GenericImage, Rgb, RgbImage};
use rusttype::{FontCollection, Scale};

pub struct Font<'a> {
    pub font: rusttype::Font<'a>,
    pub scale: Scale,
    pub color: Rgb<u8>,
}

impl<'a> Font<'a> {
    fn new(path: &Path) -> Result<Font<'a>, &'static str> {
        let mut file = fs::File::open(path).unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("Could not read font file");
        let font = FontCollection::from_bytes(buffer.to_vec())
            .into_font()
            .expect("Could not load font");
        Ok(Font {
            font: font,
            scale: Scale { x: 22.4, y: 22.4 },
            color: Rgb([255, 255, 255]),
        })
    }
}

pub struct Config<'a> {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub roi: Rect,
    pub font: Font<'a>,
}

impl<'a> Config<'a> {
    pub fn new(args: &Vec<String>) -> Result<Config<'a>, &'static str> {
        if args.len() != 6 {
            return Err("Too many or not enough arguments have been provided!");
        };
        let input_dir = Path::new(&args[1]).to_path_buf();
        let metadata = input_dir.metadata().expect("Getting Metadata failed");
        if !metadata.is_dir() {
            return Err("First argument must be a directory");
        }
        let output_dir = input_dir.join(Path::new("Output"));
        match fs::create_dir(&output_dir) {
            Ok(_) => {}
            Err(_) => return Err("Output directory already exists"),
        };

        let rect = match obtain_area(args.clone().split_off(2)) {
            Ok(a) => a,
            Err(e) => return Err(e),
        };

        let font = Font::new(Path::new("src/DejaVuSans.ttf")).unwrap();
        Ok(Config {
            input_path: input_dir,
            output_path: output_dir,
            roi: rect,
            font: font,
        })
    }
}

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

    Ok(Rgb([color[0], color[1], color[2]]))
}

fn image_paths(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
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
        _ => Err(io::Error::new(
            ErrorKind::Other,
            String::from("File: \"".to_string() + name + "\" has wrong name format"),
        )),
    }
}

fn output_file_path(target_dir: &Path, source_file: &Path) -> Result<PathBuf, io::Error> {
    let mut stem = source_file.file_stem().unwrap().to_os_string();
    stem.push("_green");
    Ok(target_dir
        .join(stem)
        .with_extension(source_file.extension().unwrap()))
    //None => Err(Error::new(ErrorKind::Other, "Problem"),
}

fn obtain_area(args: Vec<String>) -> Result<Rect, &'static str> {
    let rect: Vec<i32> = args.into_iter()
        .map(|n| n.parse().expect("Cannot convert to integer!"))
        .collect();
    Ok(Rect::at(rect[0], rect[1]).of_size(rect[2] as u32, rect[3] as u32))
}

pub fn run(config: Config) -> Result<(), Box<Error>> {
    let input_paths = image_paths(&config.input_path).expect("Could not read files in directory");
    for file in input_paths.iter() {
        let in_image = image::open(&file).expect("Opening image failed");
        let color = mean_color(&in_image, &config.roi).expect("Could not calculate mean color");
        println!("{:?}", color);

        let mut image = RgbImage::new(in_image.dimensions().0, in_image.dimensions().1);
        for p in image.pixels_mut() {
            *p = color;
        }

        let text = match citing(file.file_stem().unwrap().to_str().unwrap()) {
            Ok(c) => c,
            Err(e) => panic!("{}", e),
        };

        draw_text_mut(
            &mut image,
            config.font.color,
            10,
            10,
            config.font.scale,
            &config.font.font,
            text.as_str(),
        );

        let path = match output_file_path(&config.output_path, &file) {
            Ok(p) => p,
            Err(e) => panic!("{}", e),
        };
        let _ = image.save(path).expect("Could not save file");
    }
    Ok(())
}
