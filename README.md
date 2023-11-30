mandelbrot-rs
==============

Rusty Mandelbrot - can run sequentially or spawn multiple processes.
See [here](https://github.com/jesper-olsen/mandelbrot_erl) for an 
[Erlang](https://www.erlang.org/) version.
.

Run
-----

```
% cargo run --release -h
Usage: mandelbrot-rs [OPTIONS]

Options:
  -d, --dim <D>     pixel dimensions (width,height) [default: 1000,750]
  -x, --xrange <X>  xrange: min,max [default: -1.20,-1.0]
  -y, --yrange <Y>  yrange: min,max [default: 0.20,0.35]
  -h, --help        Print help
  -V, --version     Print version
```

```
% cargo run --release
Saving output to mandelbrot.png
```
![PNG](https://raw.githubusercontent.com/jesper-olsen/mandelbrot-rs/main/mandelbrot.png) 

Benchmark
---------

Below we will benchmark the time it takes to calculate a 25M pixel mandelbrot on a Macbook Air M1 (2020, 8 cores). All times are in seconds, and by the defaults it is the area with lower left {-1.20,0.20} and upper right {-1.0,0.35} that is mapped.

The image is calculated row by row - in multi-threaded mode 
![Rayon](https://docs.rs/rayon/latest/rayon/) farms the rows out to different threads.

```
% time cargo run --release -- --dim 5000,5000 
```

### Sequential 

| Time (real) | Time (user) | Speedup |
| ---------:  | ----------: | ------: |
| 7.7         | 7.3         |         |

### Multi-threaded 

| Time (real) | Time (user) | Speedup |
| ---------:  | ----------: | ------: |
| 1.7         | 9.6         | 4.5     |

