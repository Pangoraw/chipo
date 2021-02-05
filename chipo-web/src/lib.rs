use wasm_bindgen::prelude::*;

use chipo::compile as chipo_compile;
use chipo::emu::{Keycode, Proc};
use chipo::error::ChipoError;

// An Emulator is a wrapper for a Proc
// It can be accessed the functions _emulator(emu: &Emulator)
// instead of methods to bridge to JavaScript.
#[wasm_bindgen]
pub struct Emulator {
    proc: Proc,
}

#[wasm_bindgen]
pub fn cycle_emulator(emu: &mut Emulator) -> JsValue {
    match emu.proc.cycle() {
        Ok(chipo::emu::ProgramState::Stop) => JsValue::from("stop"),
        Ok(chipo::emu::ProgramState::Continue) => JsValue::from(0),
        Err(err) => JsValue::from(err.to_string()),
    }
}

#[wasm_bindgen]
pub fn get_display_buffer_emulator(emu: &Emulator, pixels: &mut [u8]) {
    for (i, p) in emu.proc.pixels.iter().enumerate() {
        pixels[i] = *p as u8;
    }
}

#[wasm_bindgen]
pub fn decrement_registers_emulator(emu: &mut Emulator) {
    emu.proc.decrement_registers();
}

#[wasm_bindgen]
pub fn should_buzz(emu: &Emulator) -> bool {
    emu.proc.should_buzz()
}

fn match_keycode(val: &str) -> Keycode {
    match val {
        "digit1" => Keycode::Num1,
        "digit2" => Keycode::Num2,
        "digit3" => Keycode::Num3,
        "digit4" => Keycode::Num4,
        "keya" => Keycode::A,
        "keyq" => Keycode::Q,
        "keyz" => Keycode::Z,
        "keye" => Keycode::E,
        "keyr" => Keycode::R,
        "keys" => Keycode::S,
        "keyd" => Keycode::D,
        "keyf" => Keycode::F,
        "keyw" => Keycode::W,
        "keyx" => Keycode::X,
        "keyc" => Keycode::C,
        "keyv" => Keycode::V,
        _ => Keycode::Other,
    }
}

#[wasm_bindgen]
pub fn set_key_up_emulator(emu: &mut Emulator, key: &str) {
    let keycode = match_keycode(key);
    emu.proc.set_key_up(keycode);
}

#[wasm_bindgen]
pub fn set_key_down_emulator(emu: &mut Emulator, key: &str) {
    let keycode = match_keycode(key);
    emu.proc.set_key_down(keycode);
}

#[wasm_bindgen]
pub fn new_emulator(code: &[u8]) -> Emulator {
    Emulator {
        proc: Proc::binary(code).unwrap(),
    }
}

fn convert_err(err: ChipoError) -> JsValue {
    JsValue::from_str(&err.to_string())
}

#[wasm_bindgen]
pub fn compile(code: &str, slice: &mut [u8]) -> Result<usize, JsValue> {
    let code = chipo_compile(code).map_err(convert_err)?;
    for (i, a) in code.iter().enumerate() {
        slice[i] = *a;
    }

    Ok(code.len())
}
