use wasm_bindgen::prelude::*;

use chipo::compile as chipo_compile;
use chipo::emu::Proc;
use chipo::error::{ChipoError, Result};

#[wasm_bindgen]
pub fn compile(code: &str, slice: &mut [u8]) -> usize {
    let code = chipo_compile(code.to_string()).unwrap();
    for (i, a) in code.iter().enumerate() {
        slice[i] = *a;
    }

    code.len()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
