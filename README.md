# Mandelbrot in Rust 

This repository contains an implementation for generating visualizations of the Mandelbrot set. It is part of a larger project comparing implementations across various programming languages.

The program compiles to a single native executable. It can render the Mandelbrot directly as a PNG using the image crate or produce a data file for `gnuplot` to generate a high-resolution PNG image.

## Other Language Implementations

This project compares the performance and features of Mandelbrot set generation in different languages.
Single Thread/Multi-thread shows the number of seconds it takes to do a 5000x5000 calculation.


| Language    | Repository                                                           | Single Thread   | Multi-Thread | Simd | Multi-Thread + Simd |
| :--------   | :------------------------------------------------------------------- | ---------------:| -----------: | ----:| ------------------: |
| Awk         | [mandelbrot-awk](https://github.com/jesper-olsen/mandelbrot-awk)     |           417.9 |              |      |                     |
| C           | [mandelbrot-c](https://github.com/jesper-olsen/mandelbrot-c)         |             3.6 |          0.6 |  1.1 |               0.2   |
| Erlang      | [mandelbrot_erl](https://github.com/jesper-olsen/mandelbrot_erl)     |            35.6 |          8.3 |      |                     |
| Fortran     | [mandelbrot-f](https://github.com/jesper-olsen/mandelbrot-f)         |             4.5 |              |      |                     |
| Go          | [mandelbrot-go](https://github.com/jesper-olsen/mandelbrot-go)       |             4.1 |          0.8 |  1.3 |               0.4   |
| Java        | [mandelbrot-java](https://github.com/jesper-olsen/mandelbrot-java)   |             3.9 |          0.8 |  1.4 |               0.5   |
| Lua         | [mandelbrot-lua](https://github.com/jesper-olsen/mandelbrot-lua)     |            33.2 |              |      |                     |
| Mojo        | [mandelbrot-mojo](https://github.com/jesper-olsen/mandelbrot-mojo)   |             3.8 |          1.2 |  0.7 |               0.4   |
| Nushell     | [mandelbrot-nu](https://github.com/jesper-olsen/mandelbrot-nu)       |  (est)  17186.6 |              |      |                     |
| Odin        | [mandelbrot-odin](https://github.com/jesper-olsen/mandelbrot-odin)   |             4.4 |              |      |                     |
| Python      | [mandelbrot-py](https://github.com/jesper-olsen/mandelbrot-py)       |     (pure) 93.3 | (jax)    5.9 |      |                     |
| R           | [mandelbrot-R](https://github.com/jesper-olsen/mandelbrot-R)         |           335.0 |              |      |                     |
| **Rust**    | [mandelbrot-rs](https://github.com/jesper-olsen/mandelbrot-rs)       |             4.7 |          1.3 |  1.4 |               0.8   |
| Swift       | [mandelbrot-swift](https://github.com/jesper-olsen/mandelbrot-swift) |             4.5 |          1.2 |  1.3 |               0.7   |
| Tcl         | [mandelbrot-tcl](https://github.com/jesper-olsen/mandelbrot-tcl)     |           306.9 |              |      |                     |
| Zig         | [mandelbrot-zig](https://github.com/jesper-olsen/mandelbrot-zig)     |             4.9 |          0.9 |  0.7 |               0.3   |


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

### Benchmarks
----------

Below we will benchmark the time it takes to calculate a 25M pixel mandelbrot on a Macbook Air M1 (2020, 8 cores). All times are in seconds, and by the defaults it is the area with lower left {-1.20,0.20} and upper right {-1.0,0.35} that is mapped.

The image is calculated row by row - in multi-threaded mode 
[Rayon](https://docs.rs/rayon/latest/rayon/) farms the rows out to different threads.



**Generating a 5000x5000 data file:**

```sh
time cargo run --release --bin mandelbrot -- --gnuplot --dim 5000,5000 --threads 1 > image.txt
4.17s user 0.07s system 91% cpu 4.614 total
```

**Generating a 5000x5000 data file multiple worker threads:**

```sh
 time cargo run --release --bin mandelbrot -- --gnuplot --dim 5000,5000 --threads 0 > image.txt
5.78s user 0.11s system 455% cpu 1.292 total
```

**Generating a 5000x5000 data file with SIMD and multiple worker threads:**

```sh
time cargo run --release --bin mandelbrot_simd -- --gnuplot --dim 5000,5000 --threads 1 --precision f32 > image.txt
0.95s user 0.07s system 72% cpu 1.395 total
```

**Generating a 5000x5000 data file with SIMD and multiple worker threads:**

```sh
time cargo run --release --bin mandelbrot_simd -- --gnuplot --dim 5000,5000 --threads 0 --precision f32 > image.txt
1.33s user 0.07s system 167% cpu 0.837 total
```


Hence - 3.6 x speedup
