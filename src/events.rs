use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::process;
use std::{sync::mpsc, thread, time::Duration};

use crate::State;
use daedal::create_new_thread;
use daedal::gen;
use image::{ImageBuffer, Rgb};
use sdl2::event::WindowEvent;
use std::time::SystemTime;

/// given an iterator of ImgSec, add each ImgSec to the reference imgbuf
pub fn receive_imgbuf<I>(receiver: I, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>)
where
    I: IntoIterator<Item = daedal::ImgSec>,
{
    for img in receiver {
        // iterate through all the pixels in the buffer of the image section
        for (x, y, p) in img.buf.enumerate_pixels() {
            // add the processed pixels to imgbuf with a position offset
            imgbuf.put_pixel(x + img.x, y + img.y, *p);
        }
    }
}

pub fn event_loop_singlethreaded(state: &mut State) {
    panic!("dwahawd");
    let opt = &mut state.parameters;

    let mut events = state.sdl_context.event_pump().unwrap();
    // unpack state variables
    let canvas = &mut state.canvas;
    let mut imgbuf = &mut state.imgbuf;

    let (tx, rx) = mpsc::channel();
    let imgsec = gen(rx, 0, 0, opt.size.x, opt.size.y, opt.clone());

    use sdl2::gfx::primitives::DrawRenderer;
    canvas.clear();
    for (x, y, p) in imgbuf.enumerate_pixels() {
        let image::Rgb(data) = *p;
        if data[0] > 0 || data[1] > 0 || data[2] > 0 {
            canvas
                .pixel(x as i16, y as i16, Color::RGB(data[0], data[1], data[2]))
                .unwrap();
        }
    }
    for event in events.poll_iter() {}
}

/// the main loop, sdl2 stuff and changes to the state object happen here
pub fn event_loop(state: &mut State) {
    let time = SystemTime::now();

    // unpack state variables
    let canvas = &mut state.canvas;
    let opt = &mut state.parameters;

    // then get receive any pending threads

    let (tx, rx) = &mut state.send_recieve;
    receive_imgbuf(rx.try_iter(), &mut state.imgbuf);

    // update the canvas (display imgbuf)
    use sdl2::gfx::primitives::DrawRenderer;
    for (x, y, p) in state.imgbuf.enumerate_pixels() {
        let image::Rgb(data) = *p;
        if data[0] > 0 || data[1] > 0 || data[2] > 0 {
            canvas
                .pixel(x as i16, y as i16, Color::RGB(data[0], data[1], data[2]))
                .unwrap();
        }
    }

    let mut please_stop: Option<mpsc::Sender<()>> = None;

    let mut events = state.sdl_context.event_pump().unwrap();
    for event in events.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape) | Some(Keycode::Q),
                ..
            } => process::exit(1),
            Event::Window {
                win_event: WindowEvent::Resized(..),
                ..
            } => {
                if let Some(ref ty) = please_stop {
                    ty.send(()).unwrap()
                };
                // yes this needs to be here right now
                thread::sleep(Duration::from_millis(1000));

                let size = canvas.output_size().unwrap();
                opt.size.x = size.0;
                opt.size.y = size.1;
                state.imgbuf = image::RgbImage::new(opt.size.x, opt.size.y);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } => {
                if let Some(ref ty) = please_stop {
                    ty.send(()).unwrap()
                };
                let mut z = 0.1 / (10.0_f64).powf(opt.scale);
                if z < 0.0 {
                    z = -z;
                }
                opt.position.y -= z;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } => {
                if let Some(ref ty) = please_stop {
                    ty.send(()).unwrap()
                };
                let mut z = 0.1 / (10.0_f64).powf(opt.scale);
                if z > 0.0 {
                    z = -z;
                }
                opt.position.y -= z;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Left),
                ..
            } => {
                let _ = if let Some(ref ty) = please_stop {
                    ty.send(()).unwrap()
                };
                let mut z = 0.1 / (10.0_f64).powf(opt.scale);
                if z < 0.0 {
                    z = -z;
                }
                opt.position.x -= z;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Right),
                ..
            } => {
                let _ = if let Some(ref ty) = please_stop {
                    ty.send(()).unwrap()
                };
                let mut z = 0.1 / (10.0_f64).powf(opt.scale);
                if z > 0.0 {
                    z = -z;
                }
                opt.position.x -= z;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Equals),
                ..
            } => {
                let _ = if let Some(ref ty) = please_stop {
                    ty.send(()).unwrap()
                };
                opt.scale += 0.1;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Minus),
                ..
            } => {
                let _ = if let Some(ref ty) = please_stop {
                    ty.send(()).unwrap()
                };
                opt.scale -= 0.1;
            }
            Event::KeyDown {
                keycode: Some(Keycode::P),
                ..
            } => {
                let _ = if let Some(ref ty) = please_stop {
                    ty.send(()).unwrap()
                };
                opt.iterations += 1000;
            }
            Event::KeyDown {
                keycode: Some(Keycode::N),
                ..
            } => {
                let _ = if let Some(ref ty) = please_stop {
                    ty.send(()).unwrap()
                };
                opt.iterations -= 1000;
            }
            Event::KeyDown {
                keycode: Some(Keycode::C),
                ..
            } => {
                let _ = if let Some(ref ty) = please_stop {
                    ty.send(()).unwrap()
                };
            }
            Event::KeyDown {
                keycode: Some(Keycode::S),
                ..
            } => {
                let _ = if let Some(ref ty) = please_stop {
                    ty.send(()).unwrap()
                };
                let (tx2, rx2) = mpsc::channel();
                opt.size.x *= 4;
                opt.size.y *= 4;
                eprintln!("taking printscreen....");

                eprintln!("spawning threads....");
                let mut imgbuf = image::RgbImage::new(opt.size.x, opt.size.y);
                let please_stop = Some(create_new_thread(tx2, opt.threads, opt.clone()));

                receive_imgbuf(rx2.iter().take(opt.threads as usize), &mut imgbuf);

                eprintln!("saving image....");
                let _ = if let Some(ref ty) = please_stop {
                    ty.send(()).unwrap()
                };
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
    please_stop = Some(create_new_thread(tx.clone(), opt.threads, opt.clone()));

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
