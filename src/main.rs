use clap::Parser;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use num::Complex;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    /// Use multi-threading to render
    parallel: bool,

    #[arg(short, long="dim", default_value_t = String::from("1000,750"))]
    /// Pixel dimensions (width,height)
    d: String,

    #[arg(short, long="xrange", default_value_t = String::from("-1.20,-1.0"))]
    /// X-axis range: min,max
    x: String,

    #[arg(short, long="yrange", default_value_t = String::from("0.20,0.35"))]
    /// Y-axis range: min,max
    y: String,

    #[arg(long, default_value_t = false)]
    /// Output a gnuplot data file instead of a PNG image
    gnuplot: bool,
}

/// Calculates the escape time for a point c in the complex plane.
fn escape_time(c: Complex<f64>, limit: u8) -> u8 {
    let mut z = c;
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            // Escaped
            return i;
        }
        z = z * z + c;
    }
    // Did not escape; is in the set
    limit
}

/// Writes the pixel buffer to a PNG file.
fn write_image(
    filename: &str,
    pixels: &[u8],
    bounds: (usize, usize),
) -> Result<(), image::ImageError> {
    println!("Saving PNG output to {filename}");
    let output = File::create(filename)?;
    let encoder = PngEncoder::new(output);
    encoder.write_image(
        pixels,
        bounds.0 as u32,
        bounds.1 as u32,
        ExtendedColorType::L8,
    )?;
    Ok(())
}

/// Writes the pixel buffer as a gnuplot-compatible matrix to stdout.
fn write_gnuplot_data(pixels: &[u8], bounds: (usize, usize)) -> io::Result<()> {
    let (width, _) = bounds;
    let stdout = io::stdout();
    let mut handle = BufWriter::new(stdout.lock());

    // Iterate over the pixel buffer in row-sized chunks IN REVERSE ORDER.
    // This flips the image vertically to match gnuplot's coordinate system.
    for row in pixels.chunks(width).rev() {
        let row_as_strings: Vec<String> = row.iter().map(|p| p.to_string()).collect();
        writeln!(handle, "{}", row_as_strings.join(", "))?;
    }

    Ok(())
}

/// Parses a string like "1.0,2.5" into a pair of numbers.
fn parse_number_pair<T: FromStr>(s: &str, separator: char) -> Result<(T, T), String> {
    let parts: Vec<&str> = s.split(separator).collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid format. Expected NUMBER1{separator}NUMBER2"
        ));
    }
    let first = T::from_str(parts[0]).map_err(|_| "Invalid number".to_string())?;
    let second = T::from_str(parts[1]).map_err(|_| "Invalid number".to_string())?;
    Ok((first, second))
}

/// Helper function to parse a pair and exit on error.
fn parse_pair<T: FromStr>(s: &str, label: &str) -> (T, T) {
    match parse_number_pair::<T>(s, ',') {
        Ok(pair) => pair,
        Err(msg) => {
            eprintln!("Error parsing {label}: {msg}");
            std::process::exit(1);
        }
    }
}

fn main() {
    let args = Args::parse();

    let (width, height) = parse_pair::<usize>(&args.d, "dimensions");
    let (xmin, xmax) = parse_pair::<f64>(&args.x, "xrange");
    let (ymin, ymax) = parse_pair::<f64>(&args.y, "yrange");

    let ll = Complex { re: xmin, im: ymin };
    let ur = Complex { re: xmax, im: ymax };

    let mut pixels = vec![0u8; width * height];

    let render_band = |(y, band): (usize, &mut [u8])| {
        let fheight = ur.im - ll.im;
        let fwidth = ur.re - ll.re;
        for x in 0..width {
            let c = Complex {
                re: ll.re + x as f64 * fwidth / width as f64,
                im: ur.im - y as f64 * fheight / height as f64,
            };
            band[x] = 255 - escape_time(c, 255);
        }
    };

    if args.parallel {
        pixels
            .chunks_mut(width)
            .enumerate()
            .par_bridge()
            .for_each(render_band);
    } else {
        pixels.chunks_mut(width).enumerate().for_each(render_band);
    }

    if args.gnuplot {
        write_gnuplot_data(&pixels, (width, height)).expect("Error writing gnuplot data");
    } else {
        write_image("mandelbrot.png", &pixels, (width, height)).expect("Error writing PNG file");
    }
}
