extern crate image;
extern crate imageproc;
extern crate rusttype;

mod util;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::error::Error;
use std::io::Read;
use imageproc::drawing::draw_text_mut;
use imageproc::rect::Rect;
use image::{GenericImage, Rgb, RgbImage};
use rusttype::{FontCollection, Scale};

pub struct Font<'a> {
    pub font: rusttype::Font<'a>,
    pub scale: Scale,
    pub color: Rgb<u8>,
    pub pos: (u32, u32),
}

impl<'a> Font<'a> {
    fn new(path: &Path) -> Result<Font<'a>, util::Error> {
        let mut file = fs::File::open(path)?;
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer)?;
        if let Some(font) = FontCollection::from_bytes(buffer.to_vec()).into_font() {
            Ok(Font {
                font: font,
                scale: Scale { x: 22.4, y: 22.4 },
                color: Rgb([255, 255, 255]),
                pos: (10, 10),
            })
        } else {
            Err(util::Error::Custom(String::from(
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
    pub location: String,
}

impl<'a> Config<'a> {
    pub fn new(mut args: std::env::Args) -> Result<Config<'a>, util::Error> {
        args.next();
        let input_dir = try!(
            args.next()
                .ok_or(util::Error::Custom(String::from(
                    "Cannot parse input directory"
                )))
                .map(|p| Path::new(&p).to_path_buf())
        );
        if let Ok(metadata) = input_dir.metadata() {
            if !metadata.is_dir() {
                return Err(util::Error::Custom(String::from(
                    "Input path is not a directory",
                )));
            };
        }
        let output_dir = input_dir.join(Path::new("Output"));
        try!(fs::create_dir(&output_dir));

        let location_map = try!(location_map());
        let location = try!(
            args.next()
                .ok_or(util::Error::Custom(String::from(
                    "Cannot parse location info",
                )))
                .and_then(
                    |l| location_map.get(&l).ok_or(util::Error::Custom(String::from(
                        "Given location info is unknown",
                    )))
                )
        ).clone();

        println!("Location {}", location);

        let font: Font = try!(
            args.next()
                .ok_or(util::Error::Custom(String::from("Cannot parse font path")))
                .and_then(|p| Font::new(Path::new(&p)))
        );

        let rect: Vec<i32> = args.filter_map(|n| n.parse().ok()).collect();
        if rect.len() != 4 {
            return Err(util::Error::Custom(String::from("Not enough arguments")));
        }

        Ok(Config {
            input_path: input_dir,
            output_path: output_dir,
            roi: Rect::at(rect[0], rect[1]).of_size(rect[2] as u32, rect[3] as u32),
            font: font,
            location: location,
        })
    }
}

fn location_map() -> Result<HashMap<String, String>, util::Error> {
    let mut location_map: HashMap<String, String> = HashMap::new();
    location_map.insert(String::from("MC100"), String::from("Tammela, canopy"));
    location_map.insert(String::from("MC101"), String::from("Tammela, ground"));
    location_map.insert(String::from("MC102"), String::from("Tammela, crown"));

    location_map.insert(String::from("MC103"), String::from("Punkaharju, ground"));
    location_map.insert(String::from("MC104"), String::from("Punkaharju, crown"));
    location_map.insert(String::from("MC105"), String::from("Punkaharju, landscape"));

    location_map.insert(String::from("MC106"), String::from("Hyytiälä, crown"));
    location_map.insert(String::from("MC107"), String::from("Hyytiälä, ground"));

    location_map.insert(
        String::from("MC108"),
        String::from("Sodankylä, forest, canopy"),
    );
    location_map.insert(
        String::from("MC109"),
        String::from("Sodankylä, forest, crown"),
    );
    location_map.insert(
        String::from("MC110"),
        String::from("Sodankylä, forest, ground"),
    );

    location_map.insert(
        String::from("MC111"),
        String::from("Sodankylä, wetland, ground"),
    );

    location_map.insert(String::from("MC112"), String::from("Parkano, landscape"));

    location_map.insert(String::from("MC113"), String::from("Suonenjoki, canopy"));

    location_map.insert(String::from("MC114"), String::from("Kenttärova, canopy"));
    location_map.insert(String::from("MC115"), String::from("Kenttärova, crown"));
    location_map.insert(String::from("MC116"), String::from("Kenttärova, ground"));

    location_map.insert(String::from("MC117"), String::from("Paljakka, landscape"));
    location_map.insert(String::from("MC118"), String::from("Paljakka, landscape"));
    location_map.insert(String::from("MC117-1"), String::from("Paljakka, landscape"));

    location_map.insert(String::from("MC119"), String::from("Värriö, canopy"));
    location_map.insert(String::from("MC120"), String::from("Värriö, crown"));
    location_map.insert(String::from("MC121"), String::from("Värriö, ground"));

    location_map.insert(String::from("MC122"), String::from("Lammi, crown"));
    location_map.insert(String::from("MC123"), String::from("Lammi, crown"));
    location_map.insert(String::from("MC124"), String::from("Lammi, landscape"));
    location_map.insert(String::from("MC125"), String::from("Lammi, landscape"));
    location_map.insert(String::from("MC126"), String::from("Lammi, ground"));
    location_map.insert(String::from("MC127"), String::from("Lammi, ground"));

    location_map.insert(String::from("MC128"), String::from("Kaamanen, ground"));

    location_map.insert(
        String::from("MC129"),
        String::from("Lompolojänkkä, ground"),
    );

    location_map.insert(String::from("MC130"), String::from("Tvärminne, landscape"));

    location_map.insert(String::from("MC131"), String::from("Jokioinen, landscape"));

    Ok(location_map)
}

fn crop_image(image: &mut RgbImage, rect: &Rect) -> Result<RgbImage, util::Error> {
    Ok(image
        .sub_image(
            rect.left() as u32,
            rect.top() as u32,
            rect.width(),
            rect.height(),
        )
        .to_image())
}

fn mean_color(image: &RgbImage) -> Result<Rgb<u8>, util::Error> {
    let num_pixels = image.width() * image.height();
    let color: Vec<u8> = image
        .pixels()
        .fold(vec![0u32, 0u32, 0u32], |mut acc, pixel| {
            for i in 0..acc.len() {
                acc[i] += pixel[i] as u32;
            }
            acc
        })
        .iter()
        .map(|c| (c / num_pixels) as u8)
        .collect();

    Ok(Rgb([color[0], color[1], color[2]]))
}

fn image_paths(dir: &Path) -> Result<Vec<PathBuf>, util::Error> {
    let paths: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some())
        .collect();
    Ok(paths)
}

fn parse_date(name: &str) -> Result<String, util::Error> {
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
            Ok(String::from(date + ", " + &time))
        }
        _ => Err(util::Error::Custom(String::from(
            "File: \"".to_string() + name + "\" has wrong name format",
        ))),
    }
}

fn output_file_path(target_dir: &Path, source_file: &Path) -> Result<PathBuf, util::Error> {
    let mut stem = source_file
        .file_stem()
        .ok_or_else(|| util::Error::Custom(String::from("Could not extract the file name")))?
        .to_os_string();
    stem.push("_green");
    Ok(target_dir.join(stem).with_extension(source_file
        .extension()
        .ok_or_else(|| util::Error::Custom(String::from("Could not obtain the file extension")))?))
}

pub fn run(config: Config) -> Result<(), Box<Error>> {
    let input_paths = image_paths(&config.input_path).map_err(|e| Box::new(e))?;

    for file in input_paths.iter() {
        let in_image = try!(image::open(&file));
        let color =
            try!(crop_image(&mut in_image.to_rgb(), &config.roi).and_then(|i| mean_color(&i)));
        println!("{:?}", color);

        let mut image = RgbImage::new(in_image.dimensions().0, in_image.dimensions().1);
        for p in image.pixels_mut() {
            *p = color;
        }

        let text = file.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Box::new(util::Error::Custom(String::from("Cannot obtain file name"))))
            .and_then(|n| parse_date(n).map_err(|e| Box::new(e)))?;

        draw_text_mut(
            &mut image,
            config.font.color,
            config.font.pos.0,
            config.font.pos.1,
            config.font.scale,
            &config.font.font,
            text.as_str(),
        );

        try!(
            output_file_path(&config.output_path, &file)
                .and_then(|path| image.save(path).map_err(|e| util::Error::Io(e)))
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mean_color_works() {
        use image::{Rgb, RgbImage};
        let expected = Rgb([42 as u8, 21 as u8, 84 as u8]);
        let mut image = RgbImage::new(10, 10);
        for p in image.pixels_mut() {
            *p = expected;
        }
        let actual = mean_color(&mut image).unwrap();
        assert_eq!(expected, actual);
    }
}
