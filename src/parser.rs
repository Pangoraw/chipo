use std::collections::HashMap;
use std::convert::From;
use std::num::ParseIntError;

use crate::emu::{Addr, Instruction, Vx};

#[derive(Debug, PartialEq, Eq)]
pub enum ParserError {
    NoCodeSection,
    WrongNumberOfArguments,
    WrongJumpRegister,
    UnknownSection(String),
    InstructionErr(String),
    RegisterErr(String),
    ParseIntErr(ParseIntError),
    InvalidAddress(String),
    DuplicateAddress(String),
}

impl From<ParseIntError> for ParserError {
    fn from(err: ParseIntError) -> Self {
        ParserError::ParseIntErr(err)
    }
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParserError::*;

        #[allow(unreachable_patterns)]
        let value = match self {
            NoCodeSection => "missing .code section".to_string(),
            UnknownSection(section) => format!("unknown section '{}'", section),
            InvalidAddress(address) => format!("address '{}' is invalid", address),
            DuplicateAddress(address) => format!("address '{}' has already been declared", address),
            _ => format!("unknown parsing error: {:?}", self),
        };
        f.write_str(&value)
    }
}

type Result<T> = std::result::Result<T, ParserError>;

fn parse_register(reg: &str) -> Result<Vx> {
    match reg.chars().next() {
        Some('v') => match reg.len() {
            2 => u8::from_str_radix(&reg[1..], 16)
                .map(|vx| vx as usize)
                .map_err(|_| ParserError::RegisterErr(reg.to_string())),
            _ => Err(ParserError::RegisterErr(reg.to_string())),
        },
        _ => Err(ParserError::RegisterErr(reg.to_string())),
    }
}

struct Parser<'a> {
    known_addresses: HashMap<&'a str, usize>,
    current_pointer: u32,
}

impl std::default::Default for Parser<'_> {
    fn default() -> Self {
        Parser {
            known_addresses: HashMap::new(),
            current_pointer: 0x200, // Start of code in memory space
        }
    }
}

trait FromStrRadix
where
    Self: Sized,
{
    fn from_str_radix(src: &str, radix: u32) -> Result<Self>;
}

macro_rules! impl_from_str_radix {
    ($t: ty) => {
        impl FromStrRadix for $t {
            fn from_str_radix(src: &str, radix: u32) -> Result<Self> {
                <$t>::from_str_radix(src, radix).map_err(ParserError::ParseIntErr)
            }
        }
    };
}

impl_from_str_radix!(u8);
impl_from_str_radix!(u16);
impl_from_str_radix!(u32);

fn parse_number<T>(number: &str) -> Result<T>
where
    T: FromStrRadix + std::str::FromStr<Err = std::num::ParseIntError>,
{
    if let Some(slice) = number.strip_prefix("0x") {
        T::from_str_radix(slice, 16)
    } else {
        number.parse::<T>().map_err(ParserError::ParseIntErr)
    }
}

impl<'a> Parser<'a> {
    fn parse_data_instr(&self, instruction: &'a str) -> Result<(&'a str, Vec<Instruction>)> {
        let split_pos = instruction.find(':');
        let split_pos = if let Some(pos) = split_pos {
            pos
        } else {
            return Err(ParserError::InvalidAddress(instruction.to_string()));
        };
        let name = &instruction[..split_pos];
        let instr = instruction[split_pos + 1..]
            .trim()
            .split_whitespace()
            .map(|val| parse_number(val).map(Instruction::Raw))
            .collect::<Result<Vec<Instruction>>>()?;

        Ok((name, instr))
    }

    pub fn parse_data(&mut self, instructions: &[&'a str]) -> Result<Vec<Instruction>> {
        let mut offset = 0x202;
        let mut data_instructions = vec![];
        for address in instructions.iter().filter(|line| {
            let trim = line.trim();
            !trim.is_empty() && !trim.starts_with(';')
        }) {
            let (name, mut instructions) = self.parse_data_instr(address)?;
            self.known_addresses.insert(name, offset);
            offset += 2 * instructions.len();
            data_instructions.append(&mut instructions);
        }

        let mut first_instruction = Vec::with_capacity(data_instructions.len() + 1);
        first_instruction.push(Instruction::GoTo(offset as u32));
        first_instruction.append(&mut data_instructions);

        self.current_pointer = offset as u32;

        Ok(first_instruction)
    }

    fn parse_addr(&self, symbol: &str) -> Result<Addr> {
        let address = self.known_addresses.get(symbol);

        if let Some(location) = address {
            Ok(*location as u32)
        } else {
            let parse_rel = if let Some(slice) = symbol.strip_prefix("0x") {
                i32::from_str_radix(slice, 16)
            } else {
                symbol.parse::<i32>()
            };
            if let Ok(offset) = parse_rel {
                Ok((2 * offset + self.current_pointer as i32) as u32)
            } else {
                Err(ParserError::InvalidAddress(symbol.to_string()))
            }
        }
    }

    fn parse_instr(&mut self, line: &str) -> Result<Instruction> {
        use Instruction::*;
        let ir = line.trim().to_lowercase();
        let tokens: Vec<&str> = ir.split_whitespace().collect();

        let res = match tokens[0] {
            "call" => Ok(Call(self.parse_addr(tokens[1])?)),
            "ret" => Ok(Return),
            "drw" => Ok(Disp(
                parse_register(tokens[1])?,
                parse_register(tokens[2])?,
                parse_number(tokens[3])?,
            )),
            "ld" => match tokens[1] {
                "i" => {
                    if let Ok(val) = parse_number(tokens[2]) {
                        Ok(SetAddr(val))
                    } else {
                        Ok(SetAddr(self.parse_addr(tokens[2])?))
                    }
                }
                "dt" => Ok(SetTimer(parse_register(tokens[2])?)),
                "st" => Ok(SetSoundTimer(parse_register(tokens[2])?)),
                "f" => Ok(FontLoad(parse_register(tokens[2])?)),
                "b" => Ok(BCD(parse_register(tokens[2])?)),
                "[i]" => Ok(MemDump(parse_register(tokens[2])?)),
                _ => match tokens[2] {
                    "k" => Ok(GetKeyOp(parse_register(tokens[1])?)),
                    "dt" => Ok(GetTimer(parse_register(tokens[1])?)),
                    "[i]" => Ok(MemLoad(parse_register(tokens[1])?)),
                    _ => match tokens[2].chars().next() {
                        Some('v') => Ok(SetRg(
                            parse_register(tokens[1])?,
                            parse_register(tokens[2])?,
                        )),
                        _ => Ok(Set(parse_register(tokens[1])?, parse_number(tokens[2])?)),
                    },
                },
            },
            "se" => {
                let first_register = parse_register(tokens[1])?;
                match tokens[2].chars().next() {
                    Some('v') => Ok(IfEqRg(first_register, parse_register(tokens[2])?)),
                    _ => Ok(IfEq(first_register, parse_number(tokens[2])?)),
                }
            }
            "or" => Ok(Or(parse_register(tokens[1])?, parse_register(tokens[2])?)),
            "and" => Ok(And(parse_register(tokens[1])?, parse_register(tokens[2])?)),
            "xor" => Ok(Xor(parse_register(tokens[1])?, parse_register(tokens[2])?)),
            "sne" => {
                let first_register = parse_register(tokens[1])?;
                match tokens[2].chars().next() {
                    Some('v') => Ok(IfNeqRg(first_register, parse_register(tokens[2])?)),
                    _ => Ok(IfNeq(first_register, parse_number(tokens[2])?)),
                }
            }
            "jp" => match tokens.len() {
                2 => {
                    let offset = self.parse_addr(&tokens[1])?;
                    Ok(GoTo(offset))
                }
                3 => {
                    if tokens[1] != "v0" {
                        Err(ParserError::WrongJumpRegister)
                    } else {
                        Ok(Jump(self.parse_addr(tokens[2])?))
                    }
                }
                _ => Err(ParserError::WrongNumberOfArguments),
            },
            "add" => match tokens[1] {
                "i" => Ok(AddToI(parse_register(tokens[2])?)),
                _ => match tokens[2].chars().next() {
                    Some('v') => Ok(AddRg(
                        parse_register(tokens[1])?,
                        parse_register(tokens[2])?,
                    )),
                    _ => Ok(Add(parse_register(tokens[1])?, parse_number(tokens[2])?)),
                },
            },
            "sub" => Ok(Sub(parse_register(tokens[1])?, parse_register(tokens[2])?)),
            "shr" => Ok(RightShift(parse_register(tokens[1])?)),
            "shl" => Ok(LeftShift(parse_register(tokens[1])?)),
            "cls" => Ok(DisplayClear),
            "rnd" => Ok(Rand(parse_register(tokens[1])?, parse_number(tokens[2])?)),
            "skp" => Ok(KeyOpEq(parse_register(tokens[1])?)),
            "sknp" => Ok(KeyOpNeq(parse_register(tokens[1])?)),
            _ => Err(ParserError::InstructionErr(tokens[0].to_string())),
        };

        if res.is_ok() {
            self.current_pointer += 2;
        }
        res
    }

    pub fn parse_code(&mut self, instructions: &[&'a str]) -> Result<Vec<Instruction>> {
        let mut seen = 0;
        for (i, mem) in instructions
            .iter()
            .filter(|line| !line.is_empty()) // TODO: Merge the two filters
            .enumerate()
            .filter(|(_i, line)| line.trim_end().ends_with(':'))
        {
            let addr_name = mem.trim();
            let addr_name = &addr_name[..addr_name.len() - 1];

            let res = self
                .known_addresses
                .insert(&addr_name, self.current_pointer as usize + 2 * i - seen);
            if res.is_some() {
                return Err(ParserError::DuplicateAddress(addr_name.to_string()));
            }
            seen += 2;
        }

        instructions
            .iter()
            .filter(|line| {
                let trim = line.trim();
                !trim.ends_with(':') && !trim.is_empty()
            })
            .map(|line| self.parse_instr(line))
            .collect::<Result<Vec<Instruction>>>()
    }
}

pub fn parse(program: &str) -> Result<Vec<Instruction>> {
    // Find .code section, TODO: use .data and insert instructions first
    let program = program.trim();
    let lines: Vec<&str> = program
        .split('\n')
        .filter_map(|line| {
            let trim = line.trim();
            let comment_pos = trim.find(';');
            match comment_pos {
                // inline comments
                Some(0) => None,
                Some(pos) => Some(&trim[..pos]),
                None => Some(trim),
            }
        })
        .collect();

    let mut code_sections: HashMap<&str, usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            let trim = line.trim();
            if let Some(slice) = trim.strip_prefix('.') {
                Some((i, slice))
            } else {
                None
            }
        })
        .fold(HashMap::new(), |mut acc, (i, line)| {
            acc.insert(line, i);
            acc
        });

    assert!(code_sections.len() <= 2);

    let code_section = code_sections.get("code");
    let code_section_start = if let Some(index) = code_section {
        *index
    } else {
        return Err(ParserError::NoCodeSection);
    };
    code_sections.remove("code");

    let mut parser = Parser::default();
    let data_section = code_sections.get("data");
    let mut data_section_instructions = if let Some(data_section_start) = data_section {
        let data_section_lines = &lines[*data_section_start + 1..code_section_start - 1];
        parser.parse_data(data_section_lines)?
    } else {
        if !code_sections.is_empty() {
            return Err(ParserError::UnknownSection(
                code_sections.keys().next().unwrap().to_string(),
            ));
        }
        // no data section
        vec![]
    };

    let code_section_lines = &lines[code_section_start + 1..];
    let mut code_section_instructions = parser.parse_code(code_section_lines)?;

    // TODO: Throw error if no instructions
    data_section_instructions.append(&mut code_section_instructions);
    Ok(data_section_instructions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_code_section() -> std::result::Result<(), String> {
        match parse(
            r#"
.data
; data section
        "#,
        ) {
            Err(ParserError::NoCodeSection) => Ok(()),
            other => Err(format!("expected error NoCodeSection but got {:?}", other)),
        }
    }

    #[test]
    fn test_unknown_code_section() -> std::result::Result<(), String> {
        match parse(
            r#"
.code
; code section
.other_weird_section
; should not be here
            "#,
        ) {
            Err(ParserError::UnknownSection(unkown_section)) => {
                if unkown_section == "other_weird_section" {
                    Ok(())
                } else {
                    Err(format!("{} should be other_weird_section", unkown_section))
                }
            }
            other => Err(format!("expected error UnknownSection but got {:?}", other)),
        }
    }

    #[test]
    fn test_parse_addr_invalid() -> std::result::Result<(), String> {
        match parse(
            r#"
.code
start:
    jp unkown
            "#,
        ) {
            Err(ParserError::InvalidAddress(symbol)) => {
                if symbol == "unkown" {
                    Ok(())
                } else {
                    Err(format!("{} should be unknown", symbol))
                }
            }
            other => Err(format!("expected error InvalidAddress but got {:?}", other)),
        }
    }

    #[test]
    fn test_parse_addr_symbol() -> std::result::Result<(), String> {
        let symbols = parse(
            r#"
.code
start:
    jp addr
    call addr
    ld v0 12

addr:
    jp start
            "#,
        )
        .map_err(|e| e.to_string())?;

        assert_eq!(symbols[0], Instruction::GoTo(0x206));
        assert_eq!(symbols[1], Instruction::Call(0x206)); // Calls at -2
        assert_eq!(*symbols.last().unwrap(), Instruction::GoTo(0x200));
        Ok(())
    }

    #[test]
    fn test_parse_addr_relative() -> std::result::Result<(), String> {
        let symbols = parse(
            r#"
.code
start:
    jp 2
    ret
    jp -1
            "#,
        )
        .map_err(|e| e.to_string())?;

        assert_eq!(symbols[0], Instruction::GoTo(0x204));
        assert_eq!(*symbols.last().unwrap(), Instruction::GoTo(0x202));
        Ok(())
    }

    #[test]
    fn test_parse_data_section() -> std::result::Result<(), String> {
        let symbols = parse(
            r#"
.data
addr: 0x1234

.code
start:
    jp addr
            "#,
        )
        .map_err(|e| e.to_string())?;

        assert_eq!(symbols[0], Instruction::GoTo(0x204)); // Initial jump
        assert_eq!(symbols[1], Instruction::Raw(0x1234)); // Raw data
        assert_eq!(symbols[2], Instruction::GoTo(0x202)); // Jump to addr
        Ok(())
    }

    fn test_compile(code: &str, inst: Instruction) -> std::result::Result<(), String> {
        let mut parser = Parser::default();
        let compiled = parser.parse_code(&[code]).unwrap();
        if compiled[0] == inst {
            Ok(())
        } else {
            Err(format!(
                "error: expected '{}', but got {:?}",
                code, compiled
            ))
        }
    }

    fn test_compile_to_bin(code: &str, val: u16) -> std::result::Result<(), String> {
        let mut parser = Parser::default();
        let compiled = parser.parse_code(&[code]).unwrap()[0].to_bin();
        if compiled == val {
            Ok(())
        } else {
            Err(format!(
                "error: expected 0x{:4X}, but got 0x{:4X} for {}",
                val, compiled, code
            ))
        }
    }

    #[test]
    fn test_from_asm() -> std::result::Result<(), String> {
        use Instruction::*;

        test_compile("DRW V0 V1 2", Disp(0, 1, 2))?;
        test_compile("RET", Return)?;
        test_compile("CLS", DisplayClear)?;
        test_compile("LD V0 12", Set(0, 12))?;
        test_compile("ADD I V2", AddToI(2))?;
        test_compile("ADD V1 12", Add(1, 12))?;
        test_compile("LD V4 K", GetKeyOp(4))?;
        test_compile("LD V0 0xFF", Set(0, 0xFF))?;
        test_compile("LD V0 0xFF ; a comment", Set(0, 0xFF))?;

        Ok(())
    }

    #[test]
    fn test_from_asm_to_bin() -> std::result::Result<(), String> {
        test_compile_to_bin("ADD I V2", 0xF21E)?;
        test_compile_to_bin("ADD V4 1", 0x7401)?;
        test_compile_to_bin("DRW V1 V2 5", 0xD125)?;
        test_compile_to_bin("LD [I] V1", 0xF155)?;
        test_compile_to_bin("LD V0 [I]", 0xF065)?;
        test_compile_to_bin("LD B V3", 0xF333)?;
        test_compile_to_bin("CLS", 0x00E0)?;

        Ok(())
    }

    #[test]
    fn test_parse_register_fail() {
        let expected: std::result::Result<usize, ParserError> =
            Err(ParserError::RegisterErr("vff".to_string()));
        assert_eq!(parse_register("vff"), expected);
        let expected: std::result::Result<usize, ParserError> =
            Err(ParserError::RegisterErr("v10".to_string()));
        assert_eq!(parse_register("v10"), expected);
        let expected: std::result::Result<usize, ParserError> =
            Err(ParserError::RegisterErr("123".to_string()));
        assert_eq!(parse_register("123"), expected);
    }

    #[test]
    fn test_parse_register_success() {
        let expected: std::result::Result<usize, ParserError> = Ok(3);
        assert_eq!(parse_register("v3"), expected);
        let expected: std::result::Result<usize, ParserError> = Ok(0xF);
        assert_eq!(parse_register("vf"), expected);
    }

    #[test]
    fn test_parse_number() {
        let expected: std::result::Result<u8, ParserError> = Ok(3);
        assert_eq!(parse_number("3"), expected);
        let expected: std::result::Result<u8, ParserError> = Ok(0xF);
        assert_eq!(parse_number("0x0F"), expected);
    }

    #[test]
    fn test_parse_number_fail() {
        assert!(parse_number::<u8>("v3").is_err());
        assert!(parse_number::<u8>("0xFFF").is_err()); // Overflow
        assert!(parse_number::<u16>("0xgF").is_err());
    }
}
