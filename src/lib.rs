extern crate image;
extern crate imageproc;
extern crate rusttype;

use std::path::{Path, PathBuf};
use std::{error, fmt, fs, io, num};
use std::error::Error;
use std::io::Read;
use imageproc::drawing::draw_text_mut;
use imageproc::rect::Rect;
use image::{DynamicImage, GenericImage, Rgb, RgbImage};
use rusttype::{FontCollection, Scale};

#[derive(Debug)]
pub enum CliError {
    Io(io::Error),
    Parse(num::ParseIntError),
    Custom(String),
    Else,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Io(ref err) => write!(f, "IO Error: {}", err),
            CliError::Parse(ref err) => write!(f, "Parse Errori: {}", err),
            CliError::Custom(ref err) => write!(f, "Error: {}", err),
            CliError::Else => write!(f, "Some Error"),
        }
    }
}

impl error::Error for CliError {
    fn description(&self) -> &str {
        match *self {
            CliError::Io(ref err) => err.description(),
            CliError::Parse(ref err) => err.description(),
            CliError::Custom(ref err) => err,
            CliError::Else => "Some Error",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            CliError::Io(ref err) => Some(err),
            CliError::Parse(ref err) => Some(err),
            CliError::Custom(_) => None,
            CliError::Else => None,
        }
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::Io(err)
    }
}

impl From<num::ParseIntError> for CliError {
    fn from(err: num::ParseIntError) -> CliError {
        CliError::Parse(err)
    }
}

impl From<String> for CliError {
    fn from(err: String) -> CliError {
        CliError::Custom(err)
    }
}

pub struct Font<'a> {
    pub font: rusttype::Font<'a>,
    pub scale: Scale,
    pub color: Rgb<u8>,
}

impl<'a> Font<'a> {
    fn new(path: &Path) -> Result<Font<'a>, CliError> {
        let mut file = fs::File::open(path)?;
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer)?;
        if let Some(font) = FontCollection::from_bytes(buffer.to_vec()).into_font() {
            Ok(Font {
                font: font,
                scale: Scale { x: 22.4, y: 22.4 },
                color: Rgb([255, 255, 255]),
            })
        } else {
            Err(CliError::Custom(String::from(
                "Could not obtain the file extension",
            )))
        }
    }
}

pub struct Config<'a> {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub roi: Rect,
    pub font: Font<'a>,
}

impl<'a> Config<'a> {
    pub fn new(mut args: std::env::Args) -> Result<Config<'a>, CliError> {
        args.next();
        let input_dir = try!(
            args.next()
                .ok_or(CliError::Custom(String::from(
                    "Cannot parse input directory"
                )))
                .map(|p| Path::new(&p).to_path_buf())
        );
        if let Ok(metadata) = input_dir.metadata() {
            if !metadata.is_dir() {
                return Err(CliError::Custom(String::from(
                    "Input path is not a directory",
                )));
            };
        }
        let output_dir = input_dir.join(Path::new("Output"));
        try!(fs::create_dir(&output_dir));

        let font: Font = try!(
            args.next()
                .ok_or(CliError::Custom(String::from("Cannot parse font path")))
                .and_then(|p| Font::new(Path::new(&p)))
        );
        let rect: Vec<String> = args.collect();
        if rect.len() != 4 {
            return Err(CliError::Custom(String::from("Not enough arguments")));
        }
        let rect = try!(obtain_area(rect));

        Ok(Config {
            input_path: input_dir,
            output_path: output_dir,
            roi: rect,
            font: font,
        })
    }
}

fn mean_color(img: &DynamicImage, rect: &Rect) -> Result<Rgb<u8>, CliError> {
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

fn image_paths(dir: &Path) -> Result<Vec<PathBuf>, CliError> {
    let paths: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some())
        .collect();
    Ok(paths)
}

fn citing(name: &str) -> Result<String, CliError> {
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
        _ => Err(CliError::Custom(String::from(
            "File: \"".to_string() + name + "\" has wrong name format",
        ))),
    }
}

fn output_file_path(target_dir: &Path, source_file: &Path) -> Result<PathBuf, CliError> {
    let mut stem = source_file
        .file_stem()
        .ok_or_else(|| CliError::Custom(String::from("Could not extract the file name")))?
        .to_os_string();
    stem.push("_green");
    Ok(target_dir.join(stem).with_extension(source_file
        .extension()
        .ok_or_else(|| CliError::Custom(String::from("Could not obtain the file extension")))?))
}

fn obtain_area(args: Vec<String>) -> Result<Rect, CliError> {
    let rect: Vec<i32> = args.into_iter().filter_map(|n| n.parse().ok()).collect();
    Ok(Rect::at(rect[0], rect[1]).of_size(rect[2] as u32, rect[3] as u32))
}

pub fn run(config: Config) -> Result<(), Box<Error>> {
    let input_paths = image_paths(&config.input_path).map_err(|e| Box::new(e))?;

    for file in input_paths.iter() {
        let in_image = try!(image::open(&file));
        let color = try!(mean_color(&in_image, &config.roi));
        println!("{:?}", color);

        let mut image = RgbImage::new(in_image.dimensions().0, in_image.dimensions().1);
        for p in image.pixels_mut() {
            *p = color;
        }

        let text = file.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Box::new(CliError::Custom(String::from("Cannot obtain file name"))))
            .and_then(|n| citing(n).map_err(|e| Box::new(e)))?;

        draw_text_mut(
            &mut image,
            config.font.color,
            10,
            10,
            config.font.scale,
            &config.font.font,
            text.as_str(),
        );

        try!(
            output_file_path(&config.output_path, &file)
                .and_then(|path| image.save(path).map_err(|e| CliError::Io(e)))
        );
    }
    Ok(())
}
