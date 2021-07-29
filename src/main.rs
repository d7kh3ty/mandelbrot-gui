extern crate image;
use image::{ImageBuffer, Rgb};

use std::thread::{self, JoinHandle};
use std::sync::mpsc;

use ansi_term::Colour;

/*
fn julia(a: f32, b: f32, ca: f32, cb: f32, i: i32, max: i32) -> i32 {
    //println!("a:{}, b:{}, i:{}, max:{}", a, b, i, max);
    //let f: f32 = a.powf(2.0) + b.powf(2.0);
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

fn mandel(dx: f32, dy: f32, max: i32) -> i32 {
    let mut a: f32 = 0.0;
    let mut b: f32 = 0.0;

    let mut a2: f32 = 0.0;
    let mut b2: f32 = 0.0;
    let mut i: i32 = 0;
    // f(z) = z^2 + c
    while a2 + b2 < 4.0 && i != max {
        //println!("{} + {}i", a, b);
        a2 = a * a;
        b2 = b * b;

        b = 2.0 * a * b + dy;
        a = a2 - b2 + dx;

        i = i + 1;
    }
    i
}

static IMGX: u32 = 2000;
static IMGY: u32 = 2000;

static POSX: f32 = -0.70;
static POSY: f32 = 0.26109119081845;

static ITERATIONS: i32 = 10000;

static SCALE: f32 = 100.0;

fn gen(x1: u32, x2: u32, y1: u32, y2: u32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // generate the fractal
    let mut imgbuf = image::ImageBuffer::new(IMGX, IMGY);
    for x in x1..x2 {
        print!("{:.2}%\r", (x as f32 / x2 as f32) * 100.0);
        for y in y1..y2 {
            //let dx: f32 = (x as f32 / IMGX as f32) as f32;
            //let dy: f32 = (y as f32 / IMGY as f32) as f32;
            let dx: f32 = (x as f32 - (IMGX / 2) as f32) / (SCALE * IMGX as f32) + POSX;
            let dy: f32 = (y as f32 - (IMGY / 2) as f32) / (SCALE * IMGY as f32) + POSY;

            let i = mandel(dx, dy, ITERATIONS) as u8;
            //let i = julia(-1.1086391524242, 0.25949259547294, dx, dy, 1, ITERATIONS) as u8;

            //let mut f: u8 = 0;

            //f = ((i % 100) * 255) as u8;
            //println!("{}, {}: \n i == {}, f == {}", dx, dy, i, f);

            let pixel = imgbuf.get_pixel_mut(x, y);
            //let image::Rgb(data) = *pixel;
            if i == ITERATIONS as u8 {
                *pixel = image::Rgb([0, 0, 0]);
            } else {
                *pixel = image::Rgb([i, 0, i]);
            }
        }
    }
    println!("{}, {}, {}, {}", x1, x2, y1, y2);
    println!("{}", Colour::Green.paint("done!  "));
    imgbuf
}

fn spawn(n: u32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    let mut xm: u32 = 0;
    let mut ym: u32 = 0;
    for i in 1..n / 2 {
        if n % i == 0 {
            xm = i;
            ym = n / i;
            println!("{}, {}", xm, ym);
        }
    }
    let sx = IMGX / xm;
    let sy = IMGY / ym;
    let mut threads = vec![];
    for x in 0..xm {
        for y in 0..ym {
            //println!("{}, {}, {}, {}", x * sx, x * sx + sx, y * sy, y * sy + sy);
            //gen(x * IMGX / xm, IMGX / xm, 0, IMGY / 2);
            {
                let t = tx.clone();
                threads.push(thread::spawn(move || {
                    let f = gen(x * sx, x * sx + sx, y * sy, y * sy + sy);
                    t.send(f).unwrap();
                }));
            }
        }
    }
    for thread in threads {
        thread.join().unwrap();
    }

    let mut imgbuf = image::ImageBuffer::new(IMGX, IMGY);

    for mut img in rx.iter().take(n as usize) {
        for (x, y, p) in img.enumerate_pixels_mut() {
            let pixel = imgbuf.get_pixel_mut(x, y);
            let image::Rgb(data) = *p;
            if data[0] > 0 || data[1] > 0 || data[2] > 0 {
                *pixel = *p;
            }
        }
        //imgbuf.save("assets/fractal.png").unwrap();
    }
    imgbuf
}

fn main() {
    //let mut threads = vec![];

    let imgbuf = spawn(64);
    println!("done!! saving image");
    imgbuf.save("assets/fractal.png").unwrap();
}
