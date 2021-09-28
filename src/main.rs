use mndlrs::cli::{fun::spawn,
                  structs::{ImageSize,
                            Parameters,
                            Position}};
use structopt::StructOpt;
/// A mandelbrot image generator, written in Rust!!
#[derive(StructOpt, Debug)]
#[structopt(name = "mndlrs")]
struct Opt {
    /// number of cores to run on
    #[structopt(short, long, default_value = "4")]
    cores: u32,

    /// image output location
    #[structopt(short, default_value = "fractal.png")]
    output: String,

    /// size of the image <width>x<height>
    #[structopt(long, default_value = "1024x1024")]
    size: String,

    /// define the center position of the image
    #[structopt(short, long, default_value = "-0.70,0.30")]
    position: String,

    /// zoom
    #[structopt(short, long, default_value = "7")]
    scale: f32,

    /// the number of iterations to be ran
    #[structopt(short, long, default_value = "2000")]
    iterations: u32,
}

fn new_params(size: String, position: String, scale: f32, iterations: u32) -> Parameters {
    let split = size.split('x');
    let s: Vec<&str> = split.collect();

    let sx = match s[0].parse() {
        Ok(x) => x,
        Err(e) => panic!("invalid argument to size: {}", e),
    };
    let sy = match s[1].parse() {
        Ok(x) => x,
        Err(e) => panic!("invalid argument to size: {}", e),
    };

    let split = position.split(',');
    let s: Vec<&str> = split.collect();

    println!("{},{}", s[0], s[1]);
    let px = match s[0].parse::<f32>() {
        Ok(x) => x,
        Err(e) => panic!("invalid argument to position: {}", e),
    };
    let py = match s[1].parse::<f32>() {
        Ok(y) => y,
        Err(e) => panic!("invalid argument to position: {}", e),
    };

    Parameters {
        size: ImageSize { x: sx, y: sy },
        position: Position { x: px, y: py },
        scale,
        iterations,
    }
}

fn main() {
    let opt = Opt::from_args();
    let parameters = new_params(opt.size, opt.position, opt.scale, opt.iterations);
    let imgbuf = spawn(opt.cores, parameters);
    println!("done!! saving image to: {}", opt.output);
    imgbuf.save(opt.output).unwrap();
}
