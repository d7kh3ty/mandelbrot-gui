extern crate image;

#[derive(Debug)]
pub struct ImgSec {
    pub x: u32,
    pub y: u32,
    pub buf: ImageBuffer<Rgb<u8>, Vec<u8>>,
}

use crate::options::*;

use image::{ImageBuffer, Rgb};

impl ImgSec {
    fn new(x1: u32, x2: u32, y1: u32, y2: u32) -> Self {
        Self {
            x: x1,
            y: y1,
            buf: ImageBuffer::new(x2 - x1, y2 - y1),
        }
    }
}

use std::{sync::mpsc, thread};

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

/// converts any positive integer value into RGB given a colourscheme
fn colour_bands(scheme: &[[u8; 3]], i: u32) -> image::Rgb<u8> {
    let c = (i % 256) as i32;

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
        .map(|(s, e)| (((*e as i32 - *s as i32) * c / 255) + *s as i32) as u8)
        .collect();

    image::Rgb(x.try_into().unwrap())
}

fn gen(
    recv_cancel: mpsc::Receiver<()>,
    x1: u32,
    x2: u32,
    y1: u32,
    y2: u32,
    parameters: Parameters,
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
                return img;
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
                    colour_bands(&parameters.colours, i)
                }
            })
        }
    }
    img
}

/// yes, here we spawn threads that spawn more threads, this is the
/// function you should use to generate an image section
/// make sure to give it a sender (tx), it will return a sender itself
/// that you can use to kill all child threads it spawns
pub fn create_new_thread(
    tx: mpsc::Sender<ImgSec>,
    c: u32,
    parameters: Parameters,
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

/// spawns n threads of the mandelbrot set given Parameters and sends the data on tx
fn spawn(
    tx: mpsc::Sender<ImgSec>,
    recv_cancel: mpsc::Receiver<()>,
    n: u32,
    parameters: &Parameters,
) -> Vec<std::thread::JoinHandle<()>> {
    // this whole mess is to split the image up into sections to put into threads
    let imgx = parameters.size.x;
    let imgy = parameters.size.y;
    let mut xm: u32 = 0;
    let mut ym: u32 = 0;
    let mut min = 100;
    let mut sx = imgx;
    let mut sy = imgy;
    for i in 1..n {
        if n == 1 {
            break;
        }
        if n % i == 0 && ((n / i) as i32) - 4 < min {
            xm = i;
            ym = n / i;
            min = ((n / i) as i32) - 4;
        }
        sx = imgx / xm;
        sy = imgy / ym;
    }
    // end the messy section, xm is the number of threads to divide by on x axis, ym is y axis

    // this section we spawn the threads for each section of the final image
    let mut threads = vec![];
    let mut killall = vec![];
    for x in 0..xm {
        for y in 0..ym {
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

    // wait until killswitch is recieved
    match recv_cancel.recv() {
        Ok(_) => {
            for s in killall {
                let _ = s.send(());
            }
        }
        Err(e) => eprintln!("no message recieved: {e}"),
    }
    threads
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
    /// prints output of the colour bands
    #[test]
    fn colours() {
        let scheme = vec![
            [2, 2, 11],
            [255, 4, 211],
            [0, 166, 166],
            [230, 170, 104],
            [140, 39, 30],
            [187, 222, 240],
        ];
        for i in 0..2000 {
            println!(
                "{i}: {}, {} || {:?}",
                (i % 256),
                (i / 256) % 5,
                colour_bands(&scheme, i)
            );
        }
        //assert_eq!(2 + 2, 5);
    }
}
