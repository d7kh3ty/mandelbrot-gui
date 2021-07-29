extern crate image;
use std::thread;
use std::sync::{Arc, Mutex};

use image::{ImageBuffer, Rgb};

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

static IMGX: u32 = 1000;
static IMGY: u32 = 1000;

static POSX: f32 = -0.70;
static POSY: f32 = 0.26109119081845;

static ITERATIONS: i32 = 1000;

static SCALE: f32 = 100.0;

//fn gen(x1: u32, x2: u32, y1: u32, y2: u32, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
fn gen(x1: u32, x2: u32, y1: u32, y2: u32, m: Arc<Mutex<ImageBuffer<Rgb<u8>, Vec<u8>>>>) {
    // generate the fractal
    for x in x1..x2 {
        for y in y1..y2 {
            let mut imgbuf = m.lock().unwrap();
            //let dx: f32 = (x as f32 / IMGX as f32) as f32;
            //let dy: f32 = (y as f32 / IMGY as f32) as f32;
            let dx: f32 = (x as f32 - (IMGX / 2) as f32) / (SCALE * IMGX as f32) + POSX;
            let dy: f32 = (y as f32 - (IMGY / 2) as f32) / (SCALE * IMGY as f32) + POSY;
            //let i = julia(-1.1086391524242, 0.25949259547294, dx, dy, 1, ITERATIONS) as u8;
            let i = mandel(dx, dy, ITERATIONS) as u8;

            //let mut f: u8 = 0;

            //f = ((i % 100) * 255) as u8;
            //println!("{}, {}: \n i == {}, f == {}", dx, dy, i, f);
            print!("{}%     \r", (x as f32 / x2 as f32) * 100.0);

            let pixel = imgbuf.get_pixel_mut(x, y);
            //let image::Rgb(data) = *pixel;
            if i == ITERATIONS as u8 {
                *pixel = image::Rgb([0, 0, 0]);
            } else {
                *pixel = image::Rgb([i, 0, i]);
            }
        }
    }
}

fn main() {
    // Create a new ImgBuf with width: IMGX and height: IMGY
    //let mut imgbuf = image::ImageBuffer::new(IMGX, IMGY);

    let m = Arc::new(Mutex::new(image::ImageBuffer::new(IMGX, IMGY)));
    let mut threads = vec![];

    {
        let m = Arc::clone(&m);
        threads.push(thread::spawn(move || {
            gen(0, IMGX, 0, IMGY, m);
        }));
    }
    /*
    {
        let m = Arc::clone(&m);
        threads.push(thread::spawn(move || {
            gen(IMGX / 2, IMGX, 0, IMGY / 2, m);
        }));
    }
    {
        let m = Arc::clone(&m);
        threads.push(thread::spawn(move || {
            gen(0, IMGX / 2, IMGY / 2, IMGY, m);
        }));
    }
    {
        let m = Arc::clone(&m);
        threads.push(thread::spawn(move || {
            gen(IMGX / 2, IMGX, IMGY / 2, IMGY, m);
        }));
    }*/

    for thread in threads {
        thread.join().unwrap();
    }

    let imgbuf = m.lock().unwrap();

    // Save the image as “fractal.png”, the format is deduced from the path
    imgbuf.save("assets/fractal.png").unwrap();
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_real() {
        assert_eq!(mandel(1.0, 0.0, 1.0, 1, 100), 2);
    }
    #[test]
    fn zero() {
        assert_eq!(mandel(0.0, 0.0, 1.0, 1, 100), 3);
    }
    #[test]
    fn iterative() {
        assert_eq!(mandeli(1.0, 0.0, 100), 2);
    }
    #[test]
    fn iterative_zero() {
        assert_eq!(mandeli(0.0, 0.0, 100), 3);
    }
    #[test]
    fn stays_small() {
        assert_eq!(mandel(0.0, 0.0, -1.0, 1, 10), 10);
    }
    #[test]
    fn iterative_stays_small() {
        assert_eq!(mandeli(0.0, 0.0, -1.0, 10), 10);
    }
}
*/
