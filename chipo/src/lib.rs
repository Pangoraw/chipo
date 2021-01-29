pub mod emu;
pub mod error;
mod parser;

use crate::emu::Instruction;
use crate::error::{ChipoError, Result};
use crate::parser::parse;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse() {
        let code = r#".code
  ld v0, 0x01
  cls
  drw v0, v1, 0x05
  ret"#;
        let tokens = compile(code.to_string()).unwrap();
        let res = reverse_parse(&tokens).unwrap();
        assert_eq!(res, code);
    }
}
