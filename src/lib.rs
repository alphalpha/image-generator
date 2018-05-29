extern crate conv;
extern crate image;
extern crate imageproc;
extern crate rusttype;

mod util;

use image::{GenericImage, Pixel, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use rusttype::{point, FontCollection, Point, PositionedGlyph, Scale};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct Font<'a> {
    pub font: rusttype::Font<'a>,
    pub scale: Scale,
    pub color: Rgb<u8>,
    pub pos: (u32, u32),
    pub background_color: Rgb<u8>,
}

impl<'a> Font<'a> {
    fn new(path: &Path, size: f32) -> Result<Font<'a>, util::Error> {
        let mut file = fs::File::open(path)?;
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer)?;
        if let Some(font) = FontCollection::from_bytes(buffer.to_vec()).into_font() {
            Ok(Font {
                font: font,
                scale: Scale::uniform(size),
                color: Rgb([255, 255, 255]),
                pos: (0, 0),
                background_color: Rgb([32u8, 35u8, 68u8]),
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

        let font_path = try!(
            args.next()
                .ok_or(util::Error::Custom(String::from("Cannot parse font path")))
        );

        let font_size: f32 = try!(
            args.next()
                .ok_or(util::Error::Custom(String::from("Cannot parse font size")))?
                .parse()
        );
        let font = try!(Font::new(Path::new(&font_path), font_size));

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

type LocationMap = HashMap<String, String>;
fn location_map() -> Result<LocationMap, util::Error> {
    let mut location_map: LocationMap = HashMap::new();
    location_map.insert("MC100".to_string(), "Tammela, canopy".to_string());
    location_map.insert("MC101".to_string(), "Tammela, ground".to_string());
    location_map.insert("MC102".to_string(), "Tammela, crown".to_string());

    location_map.insert("MC103".to_string(), "Punkaharju, ground".to_string());
    location_map.insert("MC104".to_string(), "Punkaharju, crown".to_string());
    location_map.insert("MC105".to_string(), "Punkaharju, landscape".to_string());

    location_map.insert("MC106".to_string(), "Hyytiälä, crown".to_string());
    location_map.insert("MC107".to_string(), "Hyytiälä, ground".to_string());

    location_map.insert("MC100".to_string(), "Tammela, canopy".to_string());
    location_map.insert("MC101".to_string(), "Tammela, ground".to_string());

    location_map.insert(
        "MC108".to_string(),
        "Sodankylä, forest, canopy".to_string(),
    );
    location_map.insert("MC109".to_string(), "Sodankylä, forest, crown".to_string());
    location_map.insert(
        "MC110".to_string(),
        "Sodankylä, forest, ground".to_string(),
    );
    location_map.insert(
        "MC111".to_string(),
        "Sodankylä, wetland, ground".to_string(),
    );

    location_map.insert("MC112".to_string(), "Parkano, landscape".to_string());

    location_map.insert("MC113".to_string(), "Suonenjoki, canopy".to_string());

    location_map.insert("MC114".to_string(), "Kenttärova, canopy".to_string());
    location_map.insert("MC115".to_string(), "Kenttärova, crown".to_string());
    location_map.insert("MC116".to_string(), "Kenttärova, ground".to_string());

    location_map.insert("MC117".to_string(), "Paljakka, landscape".to_string());
    location_map.insert("MC118".to_string(), "Paljakka, landscape".to_string());
    location_map.insert("MC117-1".to_string(), "Paljakka, landscape".to_string());

    location_map.insert("MC119".to_string(), "Värriö, canopy".to_string());
    location_map.insert("MC120".to_string(), "Värriö, crown".to_string());
    location_map.insert("MC121".to_string(), "Värriö, ground".to_string());

    location_map.insert("MC122".to_string(), "Lammi, crown".to_string());
    location_map.insert("MC123".to_string(), "Lammi, crown".to_string());
    location_map.insert("MC124".to_string(), "Lammi, landscape".to_string());
    location_map.insert("MC125".to_string(), "Lammi, landscape".to_string());
    location_map.insert("MC126".to_string(), "Lammi, ground".to_string());
    location_map.insert("MC127".to_string(), "Lammi, ground".to_string());

    location_map.insert("MC128".to_string(), "Kaamanen, ground".to_string());

    location_map.insert("MC129".to_string(), "Lompolojänkkä, ground".to_string());

    location_map.insert("MC130".to_string(), "Tvärminne, landscape".to_string());

    location_map.insert("MC131".to_string(), "Jokioinen, landscape".to_string());

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

        let dimensions = in_image.dimensions();
        let mut image = RgbImage::new(2 * dimensions.0, dimensions.1);
        for p in image
            .sub_image(0, 0, dimensions.0, dimensions.1)
            .pixels_mut()
        {
            *p.2 = color;
        }

        for p in image
            .sub_image(dimensions.0, 0, dimensions.0, dimensions.1)
            .pixels_mut()
        {
            *p.2 = in_image.get_pixel(p.0, p.1).to_rgb();
        }

        let date = file.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Box::new(util::Error::Custom(String::from("Cannot obtain file name"))))
            .and_then(|n| parse_date(n).map_err(|e| Box::new(e)))?;

        let mut position = Point {
            x: config.font.pos.0,
            y: config.font.pos.1,
        };
        let location_date = config.location.clone() + ", " + &date;
        draw_citing(&mut image, &config, &position, &location_date.as_str());

        let font_height = config.font.scale.y as u32;
        position.y = config.font.pos.1 + font_height;
        let title = "Average colour of forest activity";
        draw_citing(&mut image, &config, &position, title);

        position.y = config.font.pos.1 + 2 * font_height;
        let color_string = format!("{:?}", color);
        draw_citing(&mut image, &config, &position, &color_string.as_str());

        println!("{}, {}", location_date, color_string);
        try!(
            output_file_path(&config.output_path, &file)
                .and_then(|path| image.save(path).map_err(|e| util::Error::Io(e)))
        );
    }
    Ok(())
}

pub fn text_width(scale: Scale, font: &rusttype::Font, text: &str) -> Option<u32> {
    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);
    let glyphs: Vec<PositionedGlyph> = font.layout(text, scale, offset).collect();
    if let Some(last_glyph) = glyphs.last() {
        let mut w = last_glyph.position().x as u32;
        if let Some(bb) = last_glyph.pixel_bounding_box() {
            w = w + bb.width() as u32;
        }
        return Some(w);
    }
    None
}

fn draw_citing(image: &mut RgbImage, config: &Config, position: &Point<u32>, text: &str) {
    if let Some(width) = text_width(config.font.scale, &config.font.font, text) {
        let height = config.font.scale.y as u32;
        draw_filled_rect_mut(
            image,
            Rect::at(position.x as i32, position.y as i32).of_size(width, height),
            config.font.background_color,
        );
        draw_text_mut(
            image,
            config.font.color,
            position.x,
            position.y,
            config.font.scale,
            &config.font.font,
            text,
        );
    }
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
