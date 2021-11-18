use daedal::mandelbrot::*;
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
    #[structopt(short, long, default_value = "-0.45,0.0")]
    position: String,

    /// zoom
    #[structopt(short, long, default_value = "-0.3")]
    scale: f32,

    /// the number of iterations to be ran
    #[structopt(short, long, default_value = "200")]
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

extern crate sdl2;

use sdl2::{event::Event,
           gfx::primitives::DrawRenderer,
           keyboard::Keycode,
           pixels::Color};

pub fn main() {
    let opt = Opt::from_args();
    let mut parameters = new_params(opt.size, opt.position, opt.scale, opt.iterations);
    // let mut imgbuf = match ImageReader::open("fractal.png") {
    //     Ok(img) => match img.decode() {
    //         Ok(i) => i.to_rgb8(),
    //         Err(_) => image::RgbImage::new(imgx, imgy),
    //     },
    //     Err(_) => image::RgbImage::new(imgx, imgy),
    // };
    let mut imgbuf = image::RgbImage::new(1024, 1024);
    let (tx, rx) = mpsc::channel();
    let sender = tx.clone();
    let params = parameters.clone();
    let c = opt.cores;
    thread::spawn(move || {
        spawn(sender, c, &params);
    });
    let mut reciever = rx.try_iter();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("daedal", 1024, 1024)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    //let mut i = 0;
    use std::time::SystemTime;
    'running: loop {
        let time = SystemTime::now();
        //i = (i + 1) % 255;

        canvas.clear();
        //canvas.fill_rect(Rect::new(10, 50, 780, 580)).unwrap();
        // for i in 0..600 {
        //     for j in 0..i {
        //         canvas.pixel(j, i, Color::RGB(255, 0, 255)).unwrap();
        //     }
        // }
        //use image::io::Reader as ImageReader;
        //let mut imgbuf = match ImageReader::open("fractal.png") {
        //    Ok(img) => match img.decode() {
        //        Ok(i) => i.to_rgb8(),
        //        Err(_) => image::RgbImage::new(1024, 1024),
        //    },
        //    Err(_) => image::RgbImage::new(1024, 1024),
        //};

        if let Ok(img) = rx.try_recv() {
            println!("img section received!");
            for (x, y, p) in img.enumerate_pixels() {
                let pixel = imgbuf.get_pixel_mut(x, y);
                let image::Rgb(data) = *p;
                if data[0] > 0 || data[1] > 0 || data[2] > 0 {
                    //*pixel = image::Rgb([255, 0, 255]);
                    *pixel = *p;
                    // canvas
                    //     .pixel(x as i16, y as i16, Color::RGB(data[0], data[1], data[2]))
                    //     .unwrap();
                }
            }
            //println!("img section processed!");
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    parameters.position.x -= 0.1;
                    let sender = tx.clone();
                    let params = parameters.clone();
                    let c = opt.cores;
                    thread::spawn(move || {
                        spawn(sender, c, &params);
                    });
                }
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        println!("{}", time.elapsed().unwrap().as_millis());
        canvas
            .string(10, 10, "fuck", Color::RGB(255, 255, 255))
            .unwrap();
        canvas.present();
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
