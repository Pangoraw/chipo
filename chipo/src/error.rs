use std::convert::From;
use std::io::Error as IOError;
use std::num::ParseIntError;

use crate::emu::Instruction;
use crate::parser::ParserError;

#[derive(Debug)]
pub enum ChipoError {
    InvalidFile(String),
    ParseInstructionErr(String),
    ParseRegisterError(String),
    ParseIntError(ParseIntError),
    UnimplementedOpCodeErr(u16, Instruction),
    UnknownOpCodeErr(u16),
    ParserError(ParserError),
    IOError(IOError),
    EmptyStack,
}

pub type Result<T> = std::result::Result<T, ChipoError>;

impl std::fmt::Display for ChipoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ChipoError::*;

        #[allow(unreachable_patterns)]
        let value = match self {
            ParseInstructionErr(symbol) => format!("instruction '{}' doesn't exists", symbol),
            ParseRegisterError(register) => format!("'{}' is not a valid register", register),
            ParseIntError(err) => format!("failed to parse int: {}", err),
            EmptyStack => "the stack is empty".to_string(),
            UnimplementedOpCodeErr(instr, for_instr) => {
                format!("unimplemented OpCode 0x{:04X} {:?}", instr, for_instr)
            }
            UnknownOpCodeErr(instr) => format!("OpCode 0x{:04X} not known", instr),
            ParserError(err) => err.to_string(),
            IOError(err) => format!("io error: {}", err),
            err => format!("error: {:?}", err),
        };

        f.write_str(&value)
    }
}

macro_rules! from_err {
    ($fr: ty, $to: path) => {
        impl From<$fr> for ChipoError {
            fn from(err: $fr) -> Self {
                $to(err)
            }
        }
    };
}

from_err!(ParseIntError, ChipoError::ParseIntError);
from_err!(ParserError, ChipoError::ParserError);
from_err!(IOError, ChipoError::IOError);
