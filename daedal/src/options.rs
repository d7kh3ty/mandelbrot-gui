#[derive(Clone, Debug)]
pub struct ImageSize {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug)]
pub struct Parameters {
    pub size: ImageSize,
    pub position: Position,
    pub scale: f64,
    pub iterations: u32,
    pub threads: u32,
    pub output: String,
    pub command: Option<Command>,
    pub colours: Vec<[u8; 3]>,
}

use std::{convert::TryInto, num::ParseIntError};

use structopt::StructOpt;

fn parse_rgb(src: &str) -> Result<[u8; 3], ParseIntError> {
    println!("{src}");
    let x: Vec<u8> = src
        .split(',')
        .map(|s| match s.parse::<u8>() {
            Ok(x) => x,
            Err(e) => panic!("could not parse int {}", e),
        })
        .collect();

    println!("{x:?}");

    Ok(x.try_into().unwrap_or_else(|v: Vec<u8>| {
        panic!(
            "{} invalid.\nexpected input of 3 numbers [0-255] <r>,<g>,<b> but it was {}",
            src,
            v.len()
        )
    }))
}

/// A mandelbrot image generator, written in Rust!!
#[derive(StructOpt, Debug)]
#[structopt(name = "daedal")]
pub struct Opt {
    /// number of cores to run on
    #[structopt(short, long, default_value = "64")]
    threads: u32,

    /// image output location
    #[structopt(short, default_value = "fractal.png")]
    output: String,

    /// size of the image <width>x<height>
    #[structopt(long, default_value = "400x400")]
    size: String,

    /// define the center position of the image
    #[structopt(short, long, default_value = "-0.45,0.0")]
    position: String,

    /// zoom
    #[structopt(short, long, default_value = "-0.3")]
    scale: f64,

    /// the number of iterations to be ran
    #[structopt(short, long, default_value = "255")]
    iterations: u32,

    /// colourscheme in the format:
    /// <r>,<g>,<b> <r>,<g>,<b> ... (e.g. 10,2,4 255,0,255)
    #[structopt(short, long, parse(try_from_str = parse_rgb), default_value = "0,0,0")]
    colours: Vec<[u8; 3]>,

    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(StructOpt, Debug, Clone)]
pub enum Command {
    /// output a png of the mandelbrot set
    Screenshot {
        /// size of the image <width>x<height>
        #[structopt(short, long)]
        size: String,

        /// scale of the image
        #[structopt(short, long)]
        zoom: f64,

        output: String,
    },
    /// output a video of the mandelbrot set
    Animation {
        /// size of the animation <width>x<height>
        #[structopt(long)]
        size: String,

        /// output folder
        folder: String,

        /// start depth
        #[structopt(short, long)]
        start: f64,

        /// end depth
        #[structopt(short, long)]
        end: f64,

        /// define the center position of the image
        #[structopt(short, long)]
        position: String,

        /// increment
        #[structopt(short, default_value = "0.05")]
        inc: f64,
    },
}

impl Parameters {
    pub fn from_options() -> Self {
        let mut opt = Opt::from_args();
        println!("{opt:?}");

        match &opt.command {
            Some(Command::Screenshot { size, output, zoom }) => {
                opt.size = size.to_string();
                opt.output = output.to_string();
                opt.scale = *zoom;
            }
            Some(Command::Animation {
                size,
                folder: _,
                start: _,
                end: _,
                inc: _,
                position,
            }) => {
                opt.size = size.to_string();
                //opt.output = folder.to_string();
                opt.position = position.to_string();
            }
            None => (),
        }

        if opt.colours.len() <= 1 {
            opt.colours = vec![
                [2, 2, 11],
                [255, 97, 211],
                [0, 166, 166],
                [230, 170, 104],
                [140, 39, 30],
                [187, 222, 240],
            ];
        }

        let split = opt.size.split('x');
        let s: Vec<&str> = split.collect();

        let sx = match s[0].parse() {
            Ok(x) => x,
            Err(e) => panic!("invalid argument to size: {}", e),
        };
        let sy = match s[1].parse() {
            Ok(x) => x,
            Err(e) => panic!("invalid argument to size: {}", e),
        };

        let split = opt.position.split(',');
        let s: Vec<&str> = split.collect();

        let px = match s[0].parse::<f64>() {
            Ok(x) => x,
            Err(e) => panic!("invalid argument to position: {}", e),
        };
        let py = match s[1].parse::<f64>() {
            Ok(y) => y,
            Err(e) => panic!("invalid argument to position: {}", e),
        };

        Self {
            size: ImageSize { x: sx, y: sy },
            position: Position { x: px, y: py },
            scale: opt.scale,
            iterations: opt.iterations,
            threads: opt.threads,
            output: opt.output,
            command: opt.command,
            colours: opt.colours,
        }
    }
}
