use clap::Parser;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, ImageError};
use num::Complex;
use rayon::prelude::*;
use std::fs::File;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    //#[arg(short, long, default_value_t = false)]
    /////parallel (multi threaded)
    //p: bool,
    #[arg(short, long="dim", default_value_t = String::from("1000,750"))]
    ///pixel dimensions (width,height)
    d: String,
    #[arg(short, long="xrange", default_value_t = String::from("-1.20,-1.0"))]
    ///xrange: min,max
    x: String,
    #[arg(short, long="yrange", default_value_t = String::from("0.20,0.35"))]
    ///yrange: min,max
    y: String,
}

fn escape_time(c: Complex<f64>, limit: u8) -> u8 {
    let mut z = c; 
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return i;
        }
        z = z * z + c;
    }

    limit
}

fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize)) -> Result<(), ImageError> {
    println!("Saving output to {filename}");
    let output = File::create(filename)?;

    let encoder = PngEncoder::new(output);
    encoder.write_image(&pixels, bounds.0 as u32, bounds.1 as u32, ExtendedColorType::L8)?;

    Ok(())
}

fn parse_number_pair<T: FromStr>(s: &str, separator: char) -> Result<(T, T), &str> {
    let parts: Vec<&str> = s.split(separator).collect();

    if parts.len() != 2 {
        return Err("Invalid format. Use NUMBER1,NUMBER2");
    }

    Ok((
        T::from_str(parts[0]).map_err(|_| "Invalid number")?,
        T::from_str(parts[1]).map_err(|_| "Invalid number")?,
    ))
}

fn parse_pair<T: FromStr>(s: &str, label: &str) -> (T,T) {
    match parse_number_pair::<T>(s, ',') {
        Ok((x, y)) => (x, y),
        Err(msg) => {
            println!("failed to parse {label}: {msg}");
            std::process::exit(1)
        }
    }
}

fn main() {
    let args = Args::parse();

    let (width,height) = parse_pair::<usize>(&args.d, "dimensions");
    let (xmin,xmax) = parse_pair::<f64>(&args.x, "xrange");
    let (ymin,ymax) = parse_pair::<f64>(&args.y, "yrange");

    let ll = Complex { re: xmin, im: ymin };
    let ur = Complex { re: xmax, im: ymax };

    let mut pixels = vec![0; width * height];

    let bands: Vec<&mut [u8]> = pixels.chunks_mut(width).collect();

    bands
        //.into_par_iter()
        .into_iter()
        .enumerate()
        .for_each(|(y, band)| {
            for x in 0..width {
                let (fwidth, fheight) = (ur.re - ll.re, ur.im - ll.im);
                let c = Complex {
                    re: ll.re + x as f64 * fwidth / width as f64,
                    im: ur.im - y as f64 * fheight / height as f64,
                };

                band[x] = 255 - escape_time(c, 255);
            }
        });

    write_image("mandelbrot.png", &pixels, (width, height)).expect("error writing PNG file");
}
