extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::{AudioSubsystem, EventPump};

pub const SCALE: i32 = 10;

pub fn init() -> (WindowCanvas, EventPump, AudioSubsystem) {
    let context = sdl2::init().unwrap();
    let video_subsystem = context.video().unwrap();
    let window = video_subsystem
        .window("chip8 emulator", 64 * SCALE as u32, 32 * SCALE as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(background_color());
    canvas.clear();
    canvas.set_draw_color(foreground_color());
    canvas.present();
    let event_pump = context.event_pump().unwrap();
    let audio = context.audio().unwrap();
    (canvas, event_pump, audio)
}

pub fn clear(canvas: &mut WindowCanvas) {
    canvas.set_draw_color(background_color());
    canvas.clear();
    canvas.set_draw_color(foreground_color());
}

pub fn present(canvas: &mut WindowCanvas) {
    //canvas.clear();
    canvas.present();
}

pub fn fill_rects(canvas: &mut WindowCanvas, rects: Vec<Rect>) -> Result<(), String> {
    canvas.fill_rects(&rects)?;
    Ok(())
}

fn foreground_color() -> Color {
    Color::RGB(38, 84, 124)
}

fn background_color() -> Color {
    Color::RGB(252, 252, 252)
}

