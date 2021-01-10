mod emu;
mod error;
mod media;
mod parser;

use std::fs::read;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use crate::emu::{Instruction, Proc, ProgramState};
pub use crate::error::{ChipoError, Result};
use crate::media::audio::AudioManager;
use crate::media::screen;
use crate::parser::parse;

pub fn run_file(file: PathBuf) -> Result<()> {
    let blob = read(file)?;

    run(&blob)
}

pub fn compile(asm: String) -> Result<Vec<u8>> {
    Ok(parse(&asm)?
        .iter()
        .map(|inst| inst.to_bin())
        .flat_map(|b| vec![(b >> 8) as u8, b as u8].into_iter())
        .collect::<Vec<u8>>())
}

pub fn reverse_parse(tokens: &[u8]) -> Result<String> {
    let mut instructions = Vec::with_capacity(tokens.len() / 2);
    for i in 0..(tokens.len() / 2) {
        let val = ((tokens[i * 2] as u16) << 8) + (tokens[i * 2 + 1] as u16);
        let value = match Instruction::from(val) {
            Ok(inst) => inst.to_asm(),
            // TODO: Group raw instructions to data section
            Err(ChipoError::UnknownOpCodeErr(..)) => Instruction::Raw(val).to_asm(),
            Err(err) => {
                return Err(err);
            }
        };
        instructions.push(format!("  {}", value));
    }

    Ok(format!(".code\n{}", instructions.join("\n")))
}

pub fn run_asm(asm: String) -> Result<()> {
    let tokens = compile(asm)?;

    run(&tokens)
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
                    canvas.fill_rects(&proc.to_rects()).unwrap();
                    canvas.present();
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse() {
        let code = r#".code
  ld v0 0x01
  cls
  drw v0 v1 0x05
  ret"#;
        let tokens = compile(code.to_string()).unwrap();
        let res = reverse_parse(&tokens).unwrap();
        assert_eq!(res, code);
    }
}
