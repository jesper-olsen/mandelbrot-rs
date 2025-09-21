# Mandelbrot in Rust 

This repository contains an implementation for generating visualizations of the Mandelbrot set. It is part of a larger project comparing implementations across various programming languages.

The program compiles to a single native executable. It can render the Mandelbrot directly as a PNG using the image crate or produce a data file for `gnuplot` to generate a high-resolution PNG image.

## Other Language Implementations

This project compares the performance and features of Mandelbrot set generation in different languages.

| Language    | Repository                                                         | Single Thread   | Multi-Thread |
| :--------   | :----------------------------------------------------------------- | ---------------:| -----------: |
| Awk         | [mandelbrot-awk](https://github.com/jesper-olsen/mandelbrot-awk)     |           805.9 |              |
| **C**       | [mandelbrot-c](https://github.com/jesper-olsen/mandelbrot-c)       |             9.1 |              |
| Erlang      | [mandelbrot_erl](https://github.com/jesper-olsen/mandelbrot_erl)   |            56.0 |           16 |
| Fortran     | [mandelbrot-f](https://github.com/jesper-olsen/mandelbrot-f)       |            11.6 |              |
| Lua         | [mandelbrot-lua](https://github.com/jesper-olsen/mandelbrot-lua)   |           158.2 |              |
| Mojo        | [mandelbrot-mojo](https://github.com/jesper-olsen/mandelbrot-mojo) |                 |              |
| Nushell     | [mandelbrot-nu](https://github.com/jesper-olsen/mandelbrot-nu)     |   (est) 11488.5 |              |
| Python      | [mandelbrot-py](https://github.com/jesper-olsen/mandelbrot-py)     |    (pure) 177.2 | (jax)    7.5 |
| R           | [mandelbrot-R](https://github.com/jesper-olsen/mandelbrot-R)       |           562.0 |              |
| Rust        | [mandelbrot-rs](https://github.com/jesper-olsen/mandelbrot-rs)     |             8.9 |          2.5 |
| Tcl         | [mandelbrot-tcl](https://github.com/jesper-olsen/mandelbrot-tcl)   |           706.1 |              |


Run
-----

```
% cargo run --release -h
Usage: mandelbrot-rs [OPTIONS]

Options:
  -p, --parallel    Use multi-threading to render
  -d, --dim <D>     Pixel dimensions (width,height) [default: 1000,750]
  -x, --xrange <X>  X-axis range: min,max [default: -1.20,-1.0]
  -y, --yrange <Y>  Y-axis range: min,max [default: 0.20,0.35]
      --gnuplot     Output a gnuplot data file instead of a PNG image
  -h, --help        Print help
  -V, --version     Print version
```

```
% cargo run --release
Saving output to mandelbrot.png
```
![PNG](https://raw.githubusercontent.com/jesper-olsen/mandelbrot-rs/master/mandelbrot.png) 

Benchmarks
----------

Below we will benchmark the time it takes to calculate a 25M pixel mandelbrot on a Macbook Air M1 (2020, 8 cores). All times are in seconds, and by the defaults it is the area with lower left {-1.20,0.20} and upper right {-1.0,0.35} that is mapped.

The image is calculated row by row - in multi-threaded mode 
[Rayon](https://docs.rs/rayon/latest/rayon/) farms the rows out to different threads.



### Sequential 

```sh
% time cargo run --release -- --gnuplot --dim 5000,5000 > image.txt  
7.98s user 0.20s system 92% cpu 8.873 total
```

### Parallel  

```
cargo run --release -- --gnuplot --dim 5000,5000 --parallel > image.txt
9.95s user 0.24s system 407% cpu 2.496 total
```

Hence - 3.6 x speedup

