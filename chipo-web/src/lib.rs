use wasm_bindgen::prelude::*;

use chipo::compile as chipo_compile;
use chipo::emu::Proc;
use chipo::error::ChipoError;

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

#[wasm_bindgen]
pub fn set_keydown_emulator(_emu: &Emulator, _key: char) {}

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
