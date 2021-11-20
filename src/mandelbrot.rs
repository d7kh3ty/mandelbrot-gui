extern crate image;

#[derive(Clone)]
pub struct ImageSize {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone)]
pub struct Parameters {
    pub size:       ImageSize,
    pub position:   Position,
    pub scale:      f64,
    pub iterations: u32,
    pub threads:    u32,
    pub output:     String,
    pub command:    Option<Command>,
}

use structopt::StructOpt;

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
    #[structopt(long, default_value = "800x640")]
    size: String,

    /// define the center position of the image
    #[structopt(short, long, default_value = "-0.45,0.0")]
    position: String,

    /// zoom
    #[structopt(short, long, default_value = "-0.3")]
    scale: f64,

    /// the number of iterations to be ran
    #[structopt(short, long, default_value = "1000")]
    iterations: u32,

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
    pub fn from_opt() -> Self {
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
            size:       ImageSize { x: sx, y: sy },
            position:   Position { x: px, y: py },
            scale:      opt.scale,
            iterations: opt.iterations,
            threads:    opt.threads,
            output:     opt.output,
            command:    opt.command,
        }
    }
}

#[derive(Debug)]
pub struct ImgSec {
    pub x:   u32,
    pub y:   u32,
    pub buf: ImageBuffer<Rgb<u8>, Vec<u8>>,
}

use image::{ImageBuffer,
            Rgb};

impl ImgSec {
    fn new(x1: u32, x2: u32, y1: u32, y2: u32) -> Self {
        Self {
            x:   x1,
            y:   y1,
            buf: ImageBuffer::new(x2 - x1, y2 - y1),
        }
    }
}

use std::{sync::mpsc,
          thread};

fn mandel(dx: f64, dy: f64, max: u32) -> u32 {
    let mut a: f64 = 0.0;
    let mut b: f64 = 0.0;

    let mut a2: f64 = 0.0;
    let mut b2: f64 = 0.0;
    let mut i: u32 = 0;
    // f(z) = z^2 + c
    while a2 + b2 < 4.0 && i != max {
        //println!("{} + {}i", a, b);
        a2 = a * a;
        b2 = b * b;

        b = 2.0 * a * b + dy;
        a = a2 - b2 + dx;

        i += 1;
    }
    i
}

use std::convert::TryInto;
fn color_bands(i: u32) -> image::Rgb<u8> {
    let c: u8 = (i % 256) as u8;

    let scheme = vec![
        [2, 2, 11],
        [138, 97, 211],
        [0, 166, 166],
        [230, 170, 104],
        [140, 39, 30],
        [187, 222, 240],
    ];

    let band = (i / 256) as usize % scheme.len();
    let start = scheme[band];
    let end = if band + 1 >= scheme.len() {
        scheme[0]
    } else {
        scheme[band + 1]
    };

    let x: Vec<u8> = start
        .iter()
        .zip(end.iter())
        .map(|(s, e)| ((e - s) * c as i32 / 255 + s) as u8)
        .collect();

    image::Rgb(x.try_into().unwrap())
}

fn gen(
    recv_cancel: mpsc::Receiver<()>, x1: u32, x2: u32, y1: u32, y2: u32, parameters: Parameters,
) -> ImgSec {
    let imgx = parameters.size.x;
    let imgy = parameters.size.y;
    let posx = parameters.position.x;
    let posy = parameters.position.y;
    let scale = (10.0_f64).powf(parameters.scale);
    let iterations = parameters.iterations;
    // generate the fractal

    let mut img = ImgSec::new(x1, x2, y1, y2);

    for x in x1..x2 {
        for y in y1..y2 {
            if recv_cancel.try_recv().is_ok() {
                return img
            }
            //let dx: f64 = (x as f64 / imgx as f64) as f64;
            //let dy: f64 = (y as f64 / imgy as f64) as f64;
            let dx: f64 = (x as f64 - (imgx / 2) as f64) / (scale * imgx as f64) + posx;
            let dy: f64 = (y as f64 - (imgy / 2) as f64)
                / (scale * (imgx as f64 / imgy as f64) * imgy as f64)
                + posy;

            let i = mandel(dx, dy, iterations);
            //let i = julia(-1.1086391524242, 0.25949259547294, dx, dy, 1, iterations) as u8;

            //let mut f: u8 = 0;

            //f = ((i % 100) * 255) as u8;

            //let pixel = img.buf.get_pixel_mut(x - x1, y - y1);
            //let image::Rgb(data) = *pixel;
            img.buf.put_pixel(x - x1, y - y1, {
                if i == iterations {
                    image::Rgb([1, 1, 1])
                } else {
                    color_bands(i)
                }
            })
        }
    }
    img
}

pub fn create_new_thread(
    tx: mpsc::Sender<ImgSec>, c: u32, parameters: Parameters,
) -> mpsc::Sender<()> {
    let (please_stop, recv_cancel) = mpsc::channel();

    thread::spawn(move || {
        let threads = spawn(tx, recv_cancel, c, &parameters);
        for thread in threads {
            thread.join().unwrap();
        }
    });

    please_stop
}

fn spawn(
    tx: mpsc::Sender<ImgSec>, recv_cancel: mpsc::Receiver<()>, n: u32, parameters: &Parameters,
) -> Vec<std::thread::JoinHandle<()>> {
    // loop {
    //     thread::sleep(Duration::from_millis(500));
    //     match recv_cancel.try_recv() {
    //         Ok(_) => {
    //             println!("message received. stopping.");
    //             return
    //         }
    //         Err(TryRecvError::Disconnected) => {
    //             println!("disconnected.");
    //             return
    //         }
    //         Err(TryRecvError::Empty) => {}
    //     }
    // }
    let imgx = parameters.size.x;
    let imgy = parameters.size.y;
    let mut xm: u32 = 0;
    let mut ym: u32 = 0;
    let mut min = 100;
    let mut sx = imgx;
    let mut sy = imgy;
    for i in 1..n {
        if n == 1 {
            break
        }
        //println!("{i}/{n}");
        //println!("{n}%{i} = {}", n % i);
        if n % i == 0 {
            //println!("{ym}x{xm}, {}", ((n / i) as i32) - 4);
            if ((n / i) as i32) - 4 < min {
                xm = i;
                ym = n / i;
                min = ((n / i) as i32) - 4;
            }
            //println!("{}, {}", xm, ym);
        }
        sx = imgx / xm;
        sy = imgy / ym;
    }
    let mut threads = vec![];
    let mut killall = vec![];
    for x in 0..xm {
        for y in 0..ym {
            //println!("{}, {}, {}, {}", x * sx, x * sx + sx, y * sy, y * sy + sy);
            //gen(x * imgx / xm, imgx / xm, 0, imgy / 2);
            {
                let s = tx.clone();
                let p = parameters.clone();
                let (please_stop, recv_cancel_2) = mpsc::channel();
                threads.push(thread::spawn(move || {
                    let f = gen(recv_cancel_2, x * sx, x * sx + sx, y * sy, y * sy + sy, p);
                    match s.send(f) {
                        Ok(_) => {}
                        Err(e) => eprintln!("fuck, could not return image section: {e}"),
                    }
                }));
                killall.push(please_stop);
            }
        }
    }
    match recv_cancel.recv() {
        Ok(_) => {
            for s in killall {
                let _ = s.send(());
            }
        }
        Err(e) => eprintln!("no message recieved: {e}"),
    }
    threads
    //use image::io::Reader as ImageReader;

    // for img in rx {
    //     //println!("recieved! {recv:?}");

    //     for (x, y, p) in img.enumerate_pixels() {
    //         let pixel = (*imgbuf).get_pixel_mut(x, y);
    //         let image::Rgb(data) = *p;
    //         if data[0] > 0 || data[1] > 0 || data[2] > 0 {
    //             //*pixel = image::Rgb([255, 0, 255]);
    //             *pixel = *p;
    //         }
    //     }
    //     //imgbuf.save("fractal.png").unwrap();

    //     count -= 1;
    //     if count <= 0 {
    //         return
    //     }
    // }

    //let mut image = image::ImageBuffer::new(imgx, imgy);
    //for (x, y, p) in imgbuf.enumerate_pixels_mut() {
    //    let pixel = image.get_pixel_mut(x, y);
    //    let image::Rgb(data) = *p;
    //    if data[0] > 0 || data[1] > 0 || data[2] > 0 {
    //        *pixel = *p;
    //    }
    //    *pixel = image::Rgb([0, 0, 0]);
    //}
    //imgbuf.save("fractal.png").unwrap();

    // for (_, _, p) in imgbuf.enumerate_pixels_mut() {
    //     *p = image::Rgb([255, 255, 255]);
    // }
    // for i in 0..imgx {
    //     for j in 0..i {
    //         let pixel = imgbuf.get_pixel_mut(j, i);
    //         *pixel = image::Rgb([0, 0, 255]);
    //     }
    // }
    // imgbuf.save("fractal.png").unwrap();
    // use std::time;
    // thread::sleep(time::Duration::from_millis(4000));

    // for thread in threads {
    //     thread.join().unwrap();
    //     println!("thread received");
    //     let mut img = rx.recv().unwrap();
    //     for (x, y, p) in img.enumerate_pixels_mut() {
    //         let pixel = imgbuf.get_pixel_mut(x, y);
    //         let image::Rgb(data) = *p;
    //         if data[0] > 0 || data[1] > 0 || data[2] > 0 {
    //             *pixel = *p;
    //         }
    //         *pixel = image::Rgb([0, 0, 0]);
    //     }
    //     imgbuf.save("fractal.png").unwrap();
    //     //imgbuf.save("fractal.png").unwrap();
    //     for i in 0..imgx {
    //         for j in 0..i {
    //             let pixel = imgbuf.get_pixel_mut(j, i);
    //             *pixel = image::Rgb([255, 255, 0]);
    //         }
    //     }
    // }
}
/*
   fn julia(a: f64, b: f64, ca: f64, cb: f64, i: i32, max: i32) -> i32 {
//println!("a:{}, b:{}, i:{}, max:{}", a, b, i, max);
//let f: f64 = a.powf(2.0) + b.powf(2.0);
let a2 = a * a;
let b2 = b * b;
if a2 + b2 > 4.0 {
return i - 1;
} else if i == max {
return max;
}
julia(a2 - b2 + ca, 2.0 * a * b + cb, ca, cb, i + 1, max)
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn colors() {
        for i in 0..2000 {
            println!(
                "{i}: {}, {} || {:?}",
                (i % 256),
                (i / 256) % 5,
                color_bands(i)
            );
        }
        assert_eq!(2 + 2, 5);
    }
}
