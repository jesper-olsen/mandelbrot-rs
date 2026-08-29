//! Separate SIMD experiment binary, kept apart from `mandelbrot-rs` (the
//! main.rs binary) since Rust's std::simd (portable_simd) is still
//! nightly-only as of stable 1.98 - this uses the third-party `wide`
//! crate instead. Once std::simd stabilizes this could move back into
//! the main binary as another flag.
//!
//! Selectable between two lane widths/precisions via `--precision`:
//!   - f32x8: 8 lanes of f32, matching the width/precision tradeoff the
//!     C/Go/Java/Zig SIMD ports use (roughly double the lanes, at
//!     reduced precision near the escape boundary).
//!   - f64x4: 4 lanes of f64, same precision as the scalar/threaded
//!     binaries, just vectorized. Not directly comparable to the other
//!     languages' SIMD benchmark numbers, since they trade precision
//!     for width and this doesn't.
use clap::{Parser, ValueEnum};
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use num::Complex;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::str::FromStr;
use wide::{f32x8, f64x4};

#[derive(ValueEnum, Clone, Copy, Debug)]
enum Precision {
    /// 8 lanes of f32 (matches the other languages' SIMD ports)
    F32,
    /// 4 lanes of f64 (same precision as the scalar path)
    F64,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = 0)]
    /// Number of threads to use (0 = auto)
    pub threads: usize,

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

    #[arg(long, value_enum, default_value = "f32")]
    /// SIMD lane width/precision to use
    precision: Precision,
}

/// Calculates the escape time for a point c in the complex plane. Used
/// for the remainder columns that don't fill a full SIMD batch.
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

/// 4-wide f64 escape-time calculation, lane-for-lane identical to
/// `escape_time`: same loop structure (check the current z before
/// updating it, count how many checks passed before divergence).
///
/// Every lane keeps iterating regardless of whether it has individually
/// escaped - `undiverged` is recomputed fresh from the current z each
/// pass rather than latched with a persistent AND, since once a lane's
/// |z|^2 exceeds 4.0 the mandelbrot recurrence only grows from there.
/// Even the float edge cases (overflow to infinity, or infinity-minus-
/// infinity producing NaN) compare as "not <= 4.0", so a lane can't
/// spuriously re-enter "undiverged" once it leaves.
fn escape_time_simd_f64x4(cr: f64x4, ci: f64x4, limit: u8) -> [u8; 4] {
    let mut zr = f64x4::splat(0.0);
    let mut zi = f64x4::splat(0.0);
    let mut count = f64x4::splat(0.0);
    let four = f64x4::splat(4.0);
    let one = f64x4::splat(1.0);
    let zero = f64x4::splat(0.0);

    for _ in 0..limit {
        let zr2 = zr * zr;
        let zi2 = zi * zi;
        let undiverged = (zr2 + zi2).simd_le(four);
        if !undiverged.any() {
            break;
        }
        count += undiverged.select(one, zero);

        let new_zi = zr * zi + zr * zi + ci; // 2*zr*zi + ci
        let new_zr = zr2 - zi2 + cr;
        zr = new_zr;
        zi = new_zi;
    }

    let arr = count.round_int().to_array();
    [arr[0] as u8, arr[1] as u8, arr[2] as u8, arr[3] as u8]
}

/// 8-wide f32 escape-time calculation - same structure as
/// `escape_time_simd_f64x4`, but using f32 lanes to match the
/// precision/width tradeoff the C/Go/Java/Zig SIMD ports made.
fn escape_time_simd_f32x8(cr: f32x8, ci: f32x8, limit: u8) -> [u8; 8] {
    let mut zr = f32x8::splat(0.0);
    let mut zi = f32x8::splat(0.0);
    let mut count = f32x8::splat(0.0);
    let four = f32x8::splat(4.0);
    let one = f32x8::splat(1.0);
    let zero = f32x8::splat(0.0);

    for _ in 0..limit {
        let zr2 = zr * zr;
        let zi2 = zi * zi;
        let undiverged = (zr2 + zi2).simd_le(four);
        if !undiverged.any() {
            break;
        }
        count += undiverged.select(one, zero);

        let new_zi = zr * zi + zr * zi + ci; // 2*zr*zi + ci
        let new_zr = zr2 - zi2 + cr;
        zr = new_zr;
        zi = new_zi;
    }

    let arr = count.round_int().to_array();
    [
        arr[0] as u8,
        arr[1] as u8,
        arr[2] as u8,
        arr[3] as u8,
        arr[4] as u8,
        arr[5] as u8,
        arr[6] as u8,
        arr[7] as u8,
    ]
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
    let mut handle = BufWriter::with_capacity(65536, stdout.lock());

    for row in pixels.chunks(width).rev() {
        // Handle first element separately to manage commas
        if let Some((first, rest)) = row.split_first() {
            write!(handle, "{first}")?;
            for p in rest {
                write!(handle, ", {p}")?;
            }
        }
        writeln!(handle)?;
    }

    Ok(())
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

/// Parses a string like "1.0,2.5" into a pair of numbers.
fn parse_number_pair<T: FromStr>(s: &str, separator: char) -> Result<(T, T), String> {
    let mut iter = s.split(separator);
    let first = iter
        .next()
        .ok_or("Missing first value")?
        .parse::<T>()
        .map_err(|_| "Invalid number")?;
    let second = iter
        .next()
        .ok_or("Missing second value")?
        .parse::<T>()
        .map_err(|_| "Invalid number")?;

    if iter.next().is_some() {
        return Err("Too many values".to_string());
    }
    Ok((first, second))
}

fn main() {
    let args = Args::parse();

    if args.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global()
            .map_err(|e| {
                eprintln!("Warning: Could not set thread count: {e}");
            })
            .ok();
        println!("Rayon initialized with {} threads", args.threads);
    } else {
        println!("Rayon using default thread count (logical cores)");
    }
    println!("SIMD precision: {:?}", args.precision);

    let (width, height) = parse_pair::<usize>(&args.d, "dimensions");
    let (xmin, xmax) = parse_pair::<f64>(&args.x, "xrange");
    let (ymin, ymax) = parse_pair::<f64>(&args.y, "yrange");
    let ll = Complex { re: xmin, im: ymin };
    let ur = Complex { re: xmax, im: ymax };

    let fheight = ur.im - ll.im;
    let fwidth = ur.re - ll.re;

    let mut pixels = vec![0u8; width * height];
    pixels
        .par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, band)| {
            let im = ur.im - y as f64 * fheight / height as f64;
            match args.precision {
                Precision::F32 => {
                    let im32 = im as f32;
                    let mut x = 0;
                    while x + 8 <= width {
                        let mut re = [0f32; 8];
                        for (lane, r) in re.iter_mut().enumerate() {
                            *r = (ll.re + (x + lane) as f64 * fwidth / width as f64) as f32;
                        }
                        let cr = f32x8::new(re);
                        let ci = f32x8::splat(im32);
                        let counts = escape_time_simd_f32x8(cr, ci, 255);
                        for lane in 0..8 {
                            band[x + lane] = 255 - counts[lane];
                        }
                        x += 8;
                    }
                    while x < width {
                        let re = ll.re + x as f64 * fwidth / width as f64;
                        let c = Complex { re, im };
                        band[x] = 255 - escape_time(c, 255);
                        x += 1;
                    }
                }
                Precision::F64 => {
                    let mut x = 0;
                    while x + 4 <= width {
                        let re = [
                            ll.re + x as f64 * fwidth / width as f64,
                            ll.re + (x + 1) as f64 * fwidth / width as f64,
                            ll.re + (x + 2) as f64 * fwidth / width as f64,
                            ll.re + (x + 3) as f64 * fwidth / width as f64,
                        ];
                        let cr = f64x4::new(re);
                        let ci = f64x4::splat(im);
                        let counts = escape_time_simd_f64x4(cr, ci, 255);
                        for lane in 0..4 {
                            band[x + lane] = 255 - counts[lane];
                        }
                        x += 4;
                    }
                    while x < width {
                        let re = ll.re + x as f64 * fwidth / width as f64;
                        let c = Complex { re, im };
                        band[x] = 255 - escape_time(c, 255);
                        x += 1;
                    }
                }
            }
        });

    if args.gnuplot {
        write_gnuplot_data(&pixels, (width, height)).expect("Error writing gnuplot data");
    } else {
        write_image("mandelbrot.png", &pixels, (width, height)).expect("Error writing PNG file");
    }
}
