use clap::Parser;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder, ImageError};
use num::Complex;
use rayon::prelude::*;
use std::fs::File;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    ///parallel (multi threaded)
    p: bool,
    #[arg(short, long="dim", default_value_t = String::from("1000,750"))]
    ///pixel dimensions (width,height)
    d: String,
}

fn escape_time(c: Complex<f64>, limit: u8) -> u8 {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return i;
        }
        z = z * z + c;
    }

    limit
}

/// Parse the string `s` as a coordinate pair, like `"400x600"` or `"1.0,0.5"`.
///
/// Specifically, `s` should have the form <left><sep><right>, where <sep> is
/// the character given by the `separator` argument, and <left> and <right> are both
/// strings that can be parsed by `T::from_str`.
///
/// If `s` has the proper form, return `Some<(x, y)>`. If it doesn't parse
/// correctly, return `None`.
fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
    }
}

fn parse_number_pair<T: FromStr>(s: &str, separator: char) -> Result<(T, T), &str> {
    let parts: Vec<&str> = s.split(separator).collect();

    if parts.len() != 2 {
        return Err("Invalid format. Use NUMBER1,NUMBER2");
    }

    let first = parts[0].trim().parse::<T>().map_err(|_| "Invalid number")?;
    let second = parts[1].trim().parse::<T>().map_err(|_| "Invalid number")?;

    Ok((first, second))
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("", ','), None);
    assert_eq!(parse_pair::<i32>("10,", ','), None);
    assert_eq!(parse_pair::<i32>(",10", ','), None);
    assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x", 'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

/// Parse a pair of floating-point numbers separated by a comma as a complex
/// number.
fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None,
    }
}

#[test]
fn test_parse_complex() {
    assert_eq!(
        parse_complex("1.25,-0.0625"),
        Some(Complex {
            re: 1.25,
            im: -0.0625
        })
    );
    assert_eq!(parse_complex(",-0.0625"), None);
}

fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    ll: Complex<f64>,
    ur: Complex<f64>,
) -> Complex<f64> {
    let (width, height) = (ur.re - ll.re, ur.im - ll.im);
    Complex {
        re: ll.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: ur.im - pixel.1 as f64 * height / bounds.1 as f64,
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(
        pixel_to_point(
            (100, 200),
            (25, 175),
            Complex { re: -1.0, im: 1.0 },
            Complex { re: 1.0, im: -1.0 }
        ),
        Complex {
            re: -0.5,
            im: -0.75
        }
    );
}

fn render(pixels: &mut [u8], bounds: (usize, usize), ll: Complex<f64>, ur: Complex<f64>) {
    assert!(pixels.len() == bounds.0 * bounds.1);

    let (width, height) = bounds;
    for row in 0..height {
        for column in 0..width {
            let point = pixel_to_point(bounds, (column, row), ll, ur);
            pixels[row * width + column] = 255 - escape_time(point, 255);
        }
    }
}

fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize)) -> Result<(), ImageError> {
    let output = File::create(filename)?;

    let encoder = PngEncoder::new(output);
    encoder.write_image(&pixels, bounds.0 as u32, bounds.1 as u32, ColorType::L8)?;

    Ok(())
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args.d);
    if let Some((width, height)) = parse_pair::<usize>(&args.d, ',') {
        println!("{width} {height}");
    }

    let (width, height) = match parse_number_pair::<usize>(&args.d, ',') {
        Ok((width, height)) => (width, height),
        Err(msg) => {
            println!("failed to parse dimensions: {msg}");
            std::process::exit(1)
        }
    };

    let ll = Complex {
        re: -1.20,
        im: 0.20,
    };
    let ur = Complex {
        re: -1.00,
        im: 0.35,
    };

    let mut pixels = vec![0; width * height];

    let bands: Vec<&mut [u8]> = pixels.chunks_mut(width).collect();

    bands
        .into_par_iter()
        //bands.into_iter()
        .enumerate()
        .for_each(|(i, band)| {
            let band_ul = pixel_to_point((width, height), (0, i), ll, ur);
            let band_lr = pixel_to_point((width, height), (width, i + 1), ll, ur);
            render(band, (width, 1), band_ul, band_lr);
            //render(i, band, (width, height), ll, ur);
        });

    write_image("mandelbrot.png", &pixels, (width, height)).expect("error writing PNG file");
}
