use crate::events::receive_imgbuf;
use daedal::create_new_thread;
use daedal::options::Command;
use daedal::options::Parameters;
use image::RgbImage;
use sdl2::pixels::Color;
use std::sync::mpsc::Sender;
use std::sync::mpsc::{self, Receiver};

mod events;

/// all data required by the main loop
pub struct State {
    sdl_context: sdl2::Sdl,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    imgbuf: image::RgbImage,
    parameters: Parameters,
    kill_switch: mpsc::Sender<()>,
    send_recieve: (
        Sender<daedal::mandelbrot::ImgSec>,
        Receiver<daedal::mandelbrot::ImgSec>,
    ),
}

impl State {
    /// create a new state object, initialising any sdl2 objects that are required
    pub fn new() -> Result<Self, anyhow::Error> {
        let sdl_context = sdl2::init().unwrap();

        let options = Parameters::from_options();

        let window = sdl_context
            .video()
            .unwrap()
            .window("daedal", options.size.x, options.size.y)
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        let imgbuf = RgbImage::new(options.size.x, options.size.y);

        let send_recieve = mpsc::channel();

        let kill_switch =
            daedal::create_new_thread(send_recieve.0.clone(), options.threads, options.clone());

        Ok(Self {
            sdl_context,
            canvas,
            imgbuf,
            parameters: options,
            kill_switch,
            send_recieve,
        })
    }
}

pub fn main() {
    let mut state = State::new().unwrap();

    let mut opt = &mut state.parameters;

    #[cfg(not(target_os = "emscripten"))]
    match &opt.command {
        Some(Command::Screenshot { .. }) => {
            println!("taking screenshot......");
            let (tx, rx) = mpsc::channel();
            let kill = create_new_thread(tx, opt.threads, opt.clone());

            let mut imgbuf = RgbImage::new(opt.size.x, opt.size.y);
            receive_imgbuf(rx.iter().take(opt.threads as usize), &mut imgbuf);
            kill.send(()).unwrap();
            match imgbuf.save(&opt.output) {
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

    #[cfg(not(target_os = "emscripten"))]
    loop {
        events::event_loop(&mut state)
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
