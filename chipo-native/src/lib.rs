mod media;

use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::Keycode as SDLKeycode;
use sdl2::rect::Rect;

use chipo::{
    emu::{Keycode, Proc, ProgramState},
    error::Result,
};

use crate::media::audio::AudioManager;
use crate::media::screen;

const SCALE: i32 = 10;

fn sdl_into_chipo(kc: SDLKeycode) -> Keycode {
    match kc {
        SDLKeycode::Num1 => Keycode::Num1,
        SDLKeycode::Num2 => Keycode::Num2,
        SDLKeycode::Num3 => Keycode::Num3,
        SDLKeycode::Num4 => Keycode::Num4,
        SDLKeycode::A => Keycode::A,
        SDLKeycode::Z => Keycode::Z,
        SDLKeycode::E => Keycode::E,
        SDLKeycode::R => Keycode::R,
        SDLKeycode::Q => Keycode::Q,
        SDLKeycode::S => Keycode::S,
        SDLKeycode::D => Keycode::D,
        SDLKeycode::F => Keycode::F,
        SDLKeycode::W => Keycode::W,
        SDLKeycode::X => Keycode::X,
        SDLKeycode::C => Keycode::C,
        SDLKeycode::V => Keycode::V,
        _ => Keycode::Other,
    }
}

fn proc_to_rects(proc: &Proc) -> Vec<Rect> {
    proc.pixels
        .iter()
        .enumerate()
        .filter_map(|(pos, &b)| {
            if b {
                Some(Rect::new(
                    (pos % 64) as i32 * SCALE,
                    (pos / 64) as i32 * SCALE,
                    SCALE as u32,
                    SCALE as u32,
                ))
            } else {
                None
            }
        })
        .collect::<Vec<Rect>>()
}

pub fn run(blob: &[u8]) -> Result<()> {
    let (mut canvas, mut event_pump, mut audio) = screen::init();
    let mut audio_manager = AudioManager::init(&mut audio);
    let mut proc = Proc::binary(blob)?;

    let mut last_update = Instant::now();
    'running: loop {
        screen::clear(&mut canvas);

        match proc.cycle() {
            Ok(ProgramState::Continue) => {}
            Ok(ProgramState::Stop) => break 'running,
            Err(err) => {
                return Err(err);
            }
        }

        audio_manager.set(proc.should_buzz());
        let now = Instant::now();
        let time_since_last = now.checked_duration_since(last_update);
        if let Some(elapsed) = time_since_last {
            if elapsed > Duration::from_millis(10) {
                last_update = now;
                proc.decrement_registers();
                if proc.should_render {
                    proc.should_render = false;
                    canvas.fill_rects(&proc_to_rects(&proc)).unwrap();
                    canvas.present();
                }
            }
        }

        match event_pump.poll_event() {
            Some(Event::Quit { .. })
            | Some(Event::KeyDown {
                keycode: Some(SDLKeycode::Escape),
                ..
            }) => break 'running,
            Some(Event::KeyDown {
                keycode: Some(keycode),
                ..
            }) => proc.set_key_down(sdl_into_chipo(keycode)),
            Some(Event::KeyUp {
                keycode: Some(keycode),
                ..
            }) => proc.set_key_up(sdl_into_chipo(keycode)),
            _ => {}
        }
    }
    Ok(())
}
