use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::process;
use std::{sync::mpsc, thread, time::Duration};

use crate::State;
use daedal::create_new_thread;
use daedal::gen;
//use daedal::gen_emscripten;
use image::{ImageBuffer, Rgb};
use sdl2::event::WindowEvent;
use std::time::SystemTime;

/// given an iterator of ImgSec, add each ImgSec to the reference imgbuf
pub fn receive_imgbuf<I>(receiver: I, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) -> usize
where
    I: IntoIterator<Item = daedal::ImgSec>,
{
    let mut count = 0;
    for img in receiver {
        count += 1;
        // iterate through all the pixels in the buffer of the image section
        for (x, y, p) in img.buf.enumerate_pixels() {
            // add the processed pixels to imgbuf with a position offset
            imgbuf.put_pixel(x + img.x, y + img.y, *p);
        }
    }
    count
}

pub fn event_query(event: &sdl2::event::Event, state: &mut State) -> bool {
    let canvas = &mut state.canvas;
    let opt = &mut state.parameters;
    match event {
        Event::Quit { .. }
        | Event::KeyDown {
            keycode: Some(Keycode::Escape) | Some(Keycode::Q),
            ..
        } => {
            process::exit(1);
        }
        Event::Window {
            win_event: WindowEvent::Resized(..),
            ..
        } => {
            // yes this needs to be here right now
            thread::sleep(Duration::from_millis(1000));

            let size = canvas.output_size().unwrap();
            opt.size.x = size.0;
            opt.size.y = size.1;
            state.imgbuf = image::RgbImage::new(opt.size.x, opt.size.y);
            true
        }
        Event::KeyDown {
            keycode: Some(Keycode::Up),
            ..
        } => {
            let mut z = 0.1 / (10.0_f64).powf(opt.scale);
            if z < 0.0 {
                z = -z;
            }
            opt.position.y -= z;
            true
        }
        Event::KeyDown {
            keycode: Some(Keycode::Down),
            ..
        } => {
            let mut z = 0.1 / (10.0_f64).powf(opt.scale);
            if z > 0.0 {
                z = -z;
            }
            opt.position.y -= z;
            true
        }
        Event::KeyDown {
            keycode: Some(Keycode::Left),
            ..
        } => {
            let mut z = 0.1 / (10.0_f64).powf(opt.scale);
            if z < 0.0 {
                z = -z;
            }
            opt.position.x -= z;
            true
        }
        Event::KeyDown {
            keycode: Some(Keycode::Right),
            ..
        } => {
            let mut z = 0.1 / (10.0_f64).powf(opt.scale);
            if z > 0.0 {
                z = -z;
            }
            opt.position.x -= z;
            true
        }
        Event::KeyDown {
            keycode: Some(Keycode::Equals),
            ..
        } => {
            opt.scale += 0.1;
            true
        }
        Event::KeyDown {
            keycode: Some(Keycode::Minus),
            ..
        } => {
            opt.scale -= 0.1;
            true
        }
        Event::KeyDown {
            keycode: Some(Keycode::P),
            ..
        } => {
            opt.iterations += 255;
            true
        }
        Event::KeyDown {
            keycode: Some(Keycode::N),
            ..
        } => {
            opt.iterations -= 255;
            true
        }
        Event::KeyDown {
            keycode: Some(Keycode::S),
            ..
        } => {
            let (tx2, rx2) = mpsc::channel();
            opt.size.x *= 4;
            opt.size.y *= 4;
            eprintln!("taking printscreen....");

            eprintln!("spawning threads....");
            let mut imgbuf = image::RgbImage::new(opt.size.x, opt.size.y);
            let please_stop = Some(create_new_thread(tx2, opt.clone()));

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
            true
        }
        _ => false,
    }
}

pub fn display_set(state: &mut State) {
    println!("displaying set");
    let time = SystemTime::now();
    // unpack immutable state variables
    let opt = state.parameters.clone();
    let (_, dummy_plug) = mpsc::channel();
    //state.imgbuf = gen(dummy_plug, 0, opt.size.x, 0, opt.size.y, opt.clone()).buf;
    println!("generating fractal");
    state.imgbuf = daedal::gen_simd(dummy_plug, state.parameters.clone());

    // unpack state variables for modifying
    //let canvas = &mut state.canvas;
    // use sdl2::pixels::Color;
    // canvas.set_draw_color(Color::RGB(255, 0, 255));
    // canvas.clear();

    println!("drawing to canvas");
    use sdl2::gfx::primitives::DrawRenderer;
    for (x, y, p) in state.imgbuf.enumerate_pixels() {
        let x = x as u16;
        let y = y as u16;
        let image::Rgb(data) = *p;
        if data[0] > 0 || data[1] > 0 || data[2] > 0 {
            state
                .canvas
                .pixel(x as i16, y as i16, Color::RGB(data[0], data[1], data[2]))
                .unwrap();
        }
    }
    state
        .canvas
        .string(
            10,
            10,
            format!(
                "{}ms",
                time.elapsed().unwrap().as_millis().to_string().as_str()
            )
            .as_str(),
            Color::RGB(255, 255, 255),
        )
        .unwrap();

    println!("present");
    state.canvas.present();
}

#[cfg(target_os = "emscripten")]
pub fn event_loop_emscripten(event: sdl2::event::Event, state: &mut State) {
    if event_query(&event, state) {
        println!("caught event: {event:?}");
        display_set(state);
    }
}

pub fn display_imgbuf(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    imgbuf: &image::RgbImage,
    time: std::time::SystemTime,
) {
    // update the canvas
    use sdl2::gfx::primitives::DrawRenderer;
    for (x, y, p) in imgbuf.enumerate_pixels() {
        let image::Rgb(data) = *p;
        if data[0] > 0 || data[1] > 0 || data[2] > 0 {
            canvas
                .pixel(x as i16, y as i16, Color::RGB(data[0], data[1], data[2]))
                .unwrap();
        }
    }

    // draw time taken
    canvas
        .string(
            10,
            10,
            format!(
                "{}ms",
                time.elapsed().unwrap().as_millis().to_string().as_str()
            )
            .as_str(),
            Color::RGB(255, 255, 255),
        )
        .unwrap();

    canvas.present();
}

// the main loop, sdl2 stuff and changes to the state object happen here
// pub fn event_loop(
//     event: sdl2::event::Event,
//     state: &mut State,
//     kill_switch: &mut mpsc::Sender<()>,
// ) -> bool {
//     false
// }
