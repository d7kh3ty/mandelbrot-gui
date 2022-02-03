use std::{sync::mpsc, thread, time::Duration};

use daedal::mandelbrot::*;
use image::{ImageBuffer, Rgb, RgbImage};
use sdl2::{
    event::{Event, WindowEvent},
    gfx::primitives::DrawRenderer,
    keyboard::Keycode,
    pixels::Color,
};

/// given an iterator of ImgSec, add each ImgSec to the reference imgbuf
fn receive_imgbuf<I>(receiver: I, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>)
where
    I: IntoIterator<Item = ImgSec>,
{
    for img in receiver {
        // iterate through all the pixels in the buffer of the image section
        for (x, y, p) in img.buf.enumerate_pixels() {
            // add the processed pixels to imgbuf with a position offset
            imgbuf.put_pixel(x + img.x, y + img.y, *p);
        }
    }
}

pub fn main() {
    start();
}

fn start() {
    let mut opt = Parameters::from_options();

    match &opt.command {
        Some(Command::Screenshot { .. }) => {
            println!("taking screenshot......");
            let (tx, rx) = mpsc::channel();
            let kill = create_new_thread(tx, opt.threads, opt.clone());

            let mut imgbuf = RgbImage::new(opt.size.x, opt.size.y);
            receive_imgbuf(rx.iter().take(opt.threads as usize), &mut imgbuf);
            kill.send(()).unwrap();
            match imgbuf.save(opt.output) {
                Ok(_) => return,
                Err(e) => panic!("could not save image! {}", e),
            }
        }
        Some(Command::Animation {
            size: _,
            folder,
            start,
            end,
            inc,
            position: _,
        }) => {
            opt.scale = *start;
            opt.iterations = 4000;
            let mut count = 0;
            let total = (*end - *start) / *inc;
            while opt.scale < *end {
                count += 1;
                opt.scale += *inc;
                let (tx, rx) = mpsc::channel();
                let kill = create_new_thread(tx.clone(), opt.threads, opt.clone());

                let mut imgbuf = RgbImage::new(opt.size.x, opt.size.y);
                receive_imgbuf(rx.iter().take(opt.threads as usize), &mut imgbuf);
                kill.send(()).unwrap();
                match imgbuf.save(format!("{folder}/{count}.png")) {
                    Ok(_) => println!("image {count}/{total} saved!"),
                    Err(e) => panic!("could not save image! {}", e),
                }
            }
        }
        None => (),
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("daedal", opt.size.x, opt.size.y)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut imgbuf = RgbImage::new(opt.size.x, opt.size.y);
    let (tx, rx) = mpsc::channel();

    let mut please_stop = create_new_thread(tx.clone(), opt.threads, opt.clone());

    use std::time::SystemTime;
    canvas.clear();
    'running: loop {
        let time = SystemTime::now();

        // update the canvas to imgbuf (display imgbuf)
        canvas.clear();
        for (x, y, p) in imgbuf.enumerate_pixels() {
            let image::Rgb(data) = *p;
            if data[0] > 0 || data[1] > 0 || data[2] > 0 {
                canvas
                    .pixel(x as i16, y as i16, Color::RGB(data[0], data[1], data[2]))
                    .unwrap();
            }
        }

        // then get receive any pending threads
        receive_imgbuf(rx.try_iter(), &mut imgbuf);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape) | Some(Keycode::Q),
                    ..
                } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(..),
                    ..
                } => {
                    let _ = please_stop.send(());
                    // yes this needs to be here right now
                    thread::sleep(Duration::from_millis(1000));

                    let size = canvas.output_size().unwrap();
                    opt.size.x = size.0;
                    opt.size.y = size.1;
                    imgbuf = image::RgbImage::new(opt.size.x, opt.size.y);

                    please_stop = create_new_thread(tx.clone(), opt.threads, opt.clone())
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    let _ = please_stop.send(());
                    let mut z = 0.1 / (10.0_f64).powf(opt.scale);
                    if z < 0.0 {
                        z = -z;
                    }
                    opt.position.y -= z;
                    please_stop = create_new_thread(tx.clone(), opt.threads, opt.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    let _ = please_stop.send(());
                    let mut z = 0.1 / (10.0_f64).powf(opt.scale);
                    if z > 0.0 {
                        z = -z;
                    }
                    opt.position.y -= z;
                    please_stop = create_new_thread(tx.clone(), opt.threads, opt.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    let _ = please_stop.send(());
                    let mut z = 0.1 / (10.0_f64).powf(opt.scale);
                    if z < 0.0 {
                        z = -z;
                    }
                    opt.position.x -= z;
                    please_stop = create_new_thread(tx.clone(), opt.threads, opt.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    let _ = please_stop.send(());
                    let mut z = 0.1 / (10.0_f64).powf(opt.scale);
                    if z > 0.0 {
                        z = -z;
                    }
                    opt.position.x -= z;
                    please_stop = create_new_thread(tx.clone(), opt.threads, opt.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Equals),
                    ..
                } => {
                    let _ = please_stop.send(());
                    opt.scale += 0.1;
                    please_stop = create_new_thread(tx.clone(), opt.threads, opt.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Minus),
                    ..
                } => {
                    let _ = please_stop.send(());
                    opt.scale -= 0.1;
                    please_stop = create_new_thread(tx.clone(), opt.threads, opt.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => {
                    let _ = please_stop.send(());
                    opt.iterations += 1000;
                    please_stop = create_new_thread(tx.clone(), opt.threads, opt.clone());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::N),
                    ..
                } => {
                    let _ = please_stop.send(());
                    opt.iterations -= 1000;
                    please_stop = create_new_thread(tx.clone(), opt.threads, opt.clone());
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
                    opt.size.x *= 4;
                    opt.size.y *= 4;
                    eprintln!("taking printscreen....");

                    eprintln!("spawning threads....");
                    let mut imgbuf = image::RgbImage::new(opt.size.x, opt.size.y);
                    let please_stop = create_new_thread(tx2, opt.threads, opt.clone());

                    receive_imgbuf(rx2.iter().take(opt.threads as usize), &mut imgbuf);

                    eprintln!("saving image....");
                    let _ = please_stop.send(());
                    match imgbuf.save(format!(
                        "assets/{}-{}-{}.png",
                        opt.position.x, opt.position.y, opt.scale
                    )) {
                        Ok(_) => println!("image saved!"),
                        Err(e) => eprintln!("could not save image {e}"),
                    }
                    opt.size.x /= 4;
                    opt.size.y /= 4;
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
        let mut opt = Parameters::from_options();
        let (tx, rx) = mpsc::channel();

        opt.size.x = 800;
        opt.size.y = 640;

        let mut imgbuf = image::RgbImage::new(opt.size.x, opt.size.y);
        let please_stop = create_new_thread(tx.clone(), opt.threads, opt.clone());

        thread::sleep(Duration::from_millis(500));

        receive_imgbuf(rx.try_iter(), &mut imgbuf);
        eprintln!("saving image....");

        let _ = please_stop.send(());
        imgbuf.save("test.png").unwrap();
        assert_eq!(1, 0);
    }
}
