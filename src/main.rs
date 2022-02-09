use daedal::create_new_thread;
use daedal::options::Command;
use daedal::options::Parameters;
use image::RgbImage;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::process;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::SystemTime;

pub mod events;

/// all data required by the main loop
pub struct State {
    sdl_context: sdl2::Sdl,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    imgbuf: image::RgbImage,
    parameters: Parameters,
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

        Ok(Self {
            sdl_context,
            canvas,
            imgbuf,
            parameters: options,
            send_recieve,
        })
    }
}

#[cfg(target_os = "emscripten")]
/// run the event loop within emscripten
impl emscripten_main_loop::MainLoop for State {
    fn main_loop(&mut self) -> emscripten_main_loop::MainLoopEvent {
        let mut events = self.sdl_context.event_pump().unwrap();
        for event in events.poll_iter() {
            println!("caught event: {event:?}");
            //dbg!(sum(&[1, 2, 3, 4]));
            events::event_loop_emscripten(event, self);
        }
        emscripten_main_loop::MainLoopEvent::Continue
    }
}

fn main() {
    let mut state = State::new().unwrap();

    #[cfg(target_os = "emscripten")]
    {
        println!("hello");
        use events::display_set;
        display_set(&mut state);

        use std::process;
        emscripten_main_loop::run(state);
        process::exit(1);
    }
    let mut opt = state.parameters.clone();

    #[cfg(not(target_os = "emscripten"))]
    {
        match &opt.command {
            Some(Command::Screenshot { .. }) => {
                println!("taking screenshot......");
                let (tx, rx) = mpsc::channel();
                let kill = create_new_thread(tx, opt.clone());

                let mut imgbuf = RgbImage::new(opt.size.x, opt.size.y);
                events::receive_imgbuf(rx.iter().take(opt.threads as usize), &mut imgbuf);
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
                    let kill = create_new_thread(tx.clone(), opt.clone());

                    let mut imgbuf = RgbImage::new(opt.size.x, opt.size.y);
                    events::receive_imgbuf(rx.iter().take(opt.threads as usize), &mut imgbuf);
                    kill.send(()).unwrap();
                    match imgbuf.save(format!("{folder}/{count}.png")) {
                        Ok(_) => println!("image {count}/{total} saved!"),
                        Err(e) => panic!("could not save image! {}", e),
                    }
                }
            }
            None => (),
        }

        let (tx, rx) = &mut state.send_recieve;
        let mut kill_switch = daedal::create_new_thread(tx.clone(), opt);
        events::receive_imgbuf(rx.try_iter(), &mut state.imgbuf);

        events::display_set(&mut state);

        let mut events = state.sdl_context.event_pump().unwrap();
        let mut time = SystemTime::now();
        // the event loop
        loop {
            {
                let (_, rx) = &mut state.send_recieve;
                // receive any pending threads and display the results
                if events::receive_imgbuf(rx.try_iter(), &mut state.imgbuf) != 0 {
                    events::display_imgbuf(&mut state.canvas, &state.imgbuf, time);
                }
            }
            for event in events.poll_iter() {
                if events::event_query(&event, &mut state) {
                    time = SystemTime::now();
                    kill_switch.send(()).unwrap();
                    let (tx, rx) = &mut state.send_recieve;
                    kill_switch = create_new_thread(tx.clone(), state.parameters.clone());
                }
            }
        }
    }
}
