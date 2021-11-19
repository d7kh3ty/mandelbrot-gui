use daedal::mandelbrot::*;
use structopt::StructOpt;

/// A mandelbrot image generator, written in Rust!!
#[derive(StructOpt, Debug)]
#[structopt(name = "mndlrs")]
struct Opt {
    /// number of cores to run on
    #[structopt(short, long, default_value = "64")]
    cores: u32,

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
}

fn new_params(size: String, position: String, scale: f64, iterations: u32) -> Parameters {
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
    let px = match s[0].parse::<f64>() {
        Ok(x) => x,
        Err(e) => panic!("invalid argument to position: {}", e),
    };
    let py = match s[1].parse::<f64>() {
        Ok(y) => y,
        Err(e) => panic!("invalid argument to position: {}", e),
    };
    println!("{px},{py}");

    Parameters {
        size: ImageSize { x: sx, y: sy },
        position: Position { x: px, y: py },
        scale,
        iterations,
    }
}

use std::{sync::mpsc,
          thread,
          time::Duration};

use sdl2::{event::{Event,
                   WindowEvent},
           gfx::primitives::DrawRenderer,
           keyboard::Keycode,
           pixels::Color};
use image::{ImageBuffer,
            Rgb,
            RgbImage};

fn receive_imgbuf<I>(receiver: I, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>)
where
    I: IntoIterator<Item = ImgSec>, {
    //let mut reciever = rx.try_iter();
    for img in receiver {
        println!("img section received!");
        for (x, y, p) in img.buf.enumerate_pixels() {
            let image::Rgb(data) = *p;
            if data[0] > 0 || data[1] > 0 || data[2] > 0 {
                imgbuf.put_pixel(x + img.x, y + img.y, *p);
            }
        }
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let opt = Opt::from_args();
    let mut parameters = new_params(opt.size, opt.position, opt.scale, opt.iterations);
    let window = video_subsystem
        .window("daedal", parameters.size.x, parameters.size.y)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut imgbuf = RgbImage::new(parameters.size.x, parameters.size.y);
    let (tx, rx) = mpsc::channel();

    let mut please_stop = create_new_thread(tx.clone(), opt.cores, parameters.clone());

    use std::time::SystemTime;
    canvas.clear();
    'running: loop {
        let time = SystemTime::now();

        canvas.clear();
        for (x, y, p) in imgbuf.enumerate_pixels() {
            let image::Rgb(data) = *p;
            if data[0] > 0 || data[1] > 0 || data[2] > 0 {
                canvas
                    .pixel(x as i16, y as i16, Color::RGB(data[0], data[1], data[2]))
                    .unwrap();
            }
        }

        receive_imgbuf(rx.try_iter(), &mut imgbuf);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(..),
                    ..
                } => {
                    let _ = please_stop.send(());
                    thread::sleep(Duration::from_millis(1000));

                    let size = canvas.output_size().unwrap();
                    parameters.size.x = size.0;
                    parameters.size.y = size.1;
                    imgbuf = image::RgbImage::new(parameters.size.x, parameters.size.y);

                    please_stop = create_new_thread(tx.clone(), opt.cores, parameters.clone())
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    let _ = please_stop.send(());
                    let mut z = 0.1 / (10.0_f64).powf(parameters.scale);
                    if z < 0.0 {
                        z = -z;
                    }
                    parameters.position.y -= z;
                    please_stop = create_new_thread(tx.clone(), opt.cores, parameters.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    let _ = please_stop.send(());
                    let mut z = 0.1 / (10.0_f64).powf(parameters.scale);
                    if z > 0.0 {
                        z = -z;
                    }
                    parameters.position.y -= z;
                    please_stop = create_new_thread(tx.clone(), opt.cores, parameters.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    let _ = please_stop.send(());
                    let mut z = 0.1 / (10.0_f64).powf(parameters.scale);
                    if z < 0.0 {
                        z = -z;
                    }
                    parameters.position.x -= z;
                    please_stop = create_new_thread(tx.clone(), opt.cores, parameters.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    let _ = please_stop.send(());
                    let mut z = 0.1 / (10.0_f64).powf(parameters.scale);
                    if z > 0.0 {
                        z = -z;
                    }
                    parameters.position.x -= z;
                    please_stop = create_new_thread(tx.clone(), opt.cores, parameters.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Equals),
                    ..
                } => {
                    let _ = please_stop.send(());
                    parameters.scale += 0.1;
                    please_stop = create_new_thread(tx.clone(), opt.cores, parameters.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Minus),
                    ..
                } => {
                    let _ = please_stop.send(());
                    parameters.scale -= 0.1;
                    please_stop = create_new_thread(tx.clone(), opt.cores, parameters.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => {
                    let _ = please_stop.send(());
                    parameters.iterations += 1000;
                    please_stop = create_new_thread(tx.clone(), opt.cores, parameters.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::N),
                    ..
                } => {
                    let _ = please_stop.send(());
                    parameters.iterations -= 1000;
                    please_stop = create_new_thread(tx.clone(), opt.cores, parameters.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => {
                    let _ = please_stop.send(());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    let _ = please_stop.send(());
                    let (tx2, rx2) = mpsc::channel();
                    parameters.size.x *= 4;
                    parameters.size.y *= 4;
                    eprintln!("taking printscreen....");
                    //let end = create_new_thread(tx2, 16, parameters.clone());
                    //let (end, recv_cancel) = mpsc::channel();

                    eprintln!("spawning threads....");
                    let please_stop = create_new_thread(tx2, opt.cores, parameters.clone());
                    let mut imgbuf = image::RgbImage::new(parameters.size.x, parameters.size.y);

                    eprintln!("processing threads....");

                    receive_imgbuf(rx2.iter().take(opt.cores as usize), &mut imgbuf);

                    eprintln!("saving image....");
                    let _ = please_stop.send(());
                    match imgbuf.save(opt.output.clone()) {
                        Ok(_) => println!("image saved!"),
                        Err(e) => eprintln!("could not save image {e}"),
                    }
                }
                _ => {}
            }
        }

        //println!("{:?}", size);
        canvas
            .string(
                10,
                10,
                time.elapsed().unwrap().as_millis().to_string().as_str(),
                Color::RGB(255, 255, 255),
            )
            .unwrap();

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dynamic_resolution() {
        let opt = Opt::from_args();
        let mut parameters = new_params(opt.size, opt.position, opt.scale, opt.iterations);
        let (tx, rx) = mpsc::channel();

        parameters.size.x = 800;
        parameters.size.y = 640;

        let mut imgbuf = image::RgbImage::new(parameters.size.x, parameters.size.y);
        let please_stop = create_new_thread(tx.clone(), opt.cores, parameters.clone());

        thread::sleep(Duration::from_millis(500));

        receive_imgbuf(rx.try_iter(), &mut imgbuf);
        eprintln!("saving image....");

        let _ = please_stop.send(());
        imgbuf.save("test.png").unwrap();
        assert_eq!(1, 0);
    }
}
