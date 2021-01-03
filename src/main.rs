#![feature(duration_zero)]

mod core;
mod emu;
mod media;

use std::time::{Duration, Instant};

use clap::{App, Arg};
use colorful::Colorful;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use emu::proc::Proc;
use media::audio::AudioManager;
use media::screen;

fn main() {
    let matches = App::new("Pico")
        .version("1.0")
        .author("Pangoraw")
        .about("A chip8 emulator")
        .arg(
            Arg::with_name("file")
                .short("f")
                .value_name("FILE")
                .help("The file to execute")
                .required(true)
                .takes_value(true),
        )
        .get_matches();
    let file = matches.value_of("file").unwrap();

    match run(file) {
        Ok(()) => println!("Pico exited nicely!"),
        Err(msg) => println!("{}", msg.red()),
    }
}

fn run(file: &str) -> Result<(), String> {
    let (mut canvas, mut event_pump, mut audio) = screen::init();
    let mut audio_manager = AudioManager::init(&mut audio);
    let mut proc = Proc::new(file.to_string()).unwrap();

    let mut last_update = Instant::now();
    let delta_time = Duration::from_millis(16);
    'running: loop {
        screen::clear(&mut canvas);

        proc.cycle()?;
        audio_manager.set(proc.should_buzz());

        // TODO: investigate frame updates specs
        let now = Instant::now();
        let time_to_wait = delta_time.checked_sub(now - last_update);
        if time_to_wait.is_none() {
            screen::fill_rects(&mut canvas, proc.to_rects())?;
            screen::present(&mut canvas);
            last_update = Instant::now();
        }

        match event_pump.poll_event() {
            Some(Event::Quit { .. })
            | Some(Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            }) => break 'running,
            Some(Event::KeyDown {
                keycode: Some(keycode),
                ..
            }) => proc.set_key_down(keycode),
            Some(Event::KeyUp {
                keycode: Some(keycode),
                ..
            }) => proc.set_key_up(keycode),
            _ => {}
        }

    }
    Ok(())
}
