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
    scale: f64,

    /// the number of iterations to be ran
    #[structopt(short, long, default_value = "200")]
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

use sdl2::{event::Event,
           gfx::primitives::DrawRenderer,
           keyboard::Keycode,
           pixels::Color};

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("daedal", 1024, 1024)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

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

    let mut please_stop = create_new_thread(tx.clone(), opt.cores, parameters.clone());
    //let mut i = 0;
    use std::time::SystemTime;
    canvas.clear();
    'running: loop {
        let size = canvas.output_size().unwrap();
        parameters.size.x = size.0;
        parameters.size.y = size.1;

        let time = SystemTime::now();
        //i = (i + 1) % 255;

        canvas.clear();
        for (x, y, p) in imgbuf.enumerate_pixels() {
            //let pixel = imgbuf.get_pixel_mut(x, y);
            let image::Rgb(data) = *p;
            if data[0] > 0 || data[1] > 0 || data[2] > 0 {
                //*pixel = image::Rgb([255, 0, 255]);
                //*pixel = *p;
                canvas
                    .pixel(x as i16, y as i16, Color::RGB(data[0], data[1], data[2]))
                    .unwrap();
            }
        }
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

        loop {
            if let Ok(img) = rx.try_recv() {
                println!("img section received!");
                for (x, y, p) in img.enumerate_pixels() {
                    //let pixel = imgbuf.get_pixel_mut(x, y);
                    let image::Rgb(data) = *p;
                    if data[0] > 0 || data[1] > 0 || data[2] > 0 {
                        //*pixel = image::Rgb([255, 0, 255]);
                        //*pixel = *p;
                        imgbuf.put_pixel(x, y, *p);
                        canvas
                            .pixel(x as i16, y as i16, Color::RGB(data[0], data[1], data[2]))
                            .unwrap();
                    }
                }
                //println!("img section processed!");
            } else {
                println!("none recieved");
                break
            }
        }

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
                    parameters.size.x = 4086;
                    parameters.size.y = 4086;
                    //let end = create_new_thread(tx2, 16, parameters.clone());
                    let (end, recv_cancel) = mpsc::channel();

                    let c = opt.cores;
                    let params = parameters.clone();

                    let threads = spawn(tx2, recv_cancel, c, &params);

                    for thread in threads {
                        thread.join().unwrap();
                    }
                    let mut imgbuf = image::RgbImage::new(parameters.size.x, parameters.size.y);

                    for mut img in rx2.iter().take(opt.cores as usize) {
                        println!("img section received!");
                        for (x, y, p) in img.enumerate_pixels() {
                            let pixel = imgbuf.get_pixel_mut(x, y);
                            let image::Rgb(data) = *p;
                            if data[0] > 0 || data[1] > 0 || data[2] > 0 {
                                //*pixel = image::Rgb([255, 0, 255]);
                                *pixel = *p;
                            }
                        }
                    }
                    let _ = end.send(());
                    match imgbuf.save(opt.output.clone()) {
                        Ok(_) => return,
                        Err(e) => eprintln!("could not save image {e}"),
                    }

                    loop {
                        println!("waiting for image section");
                        //println!("img section processed!");
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
