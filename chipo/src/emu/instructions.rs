use crate::error::{ChipoError, Result};

pub type Addr = u32;
pub type Vx = usize;
pub type Val = u8;

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Raw(u16), // Raw data used for the data section
    CallPrg(Addr),
    DisplayClear,
    Return,
    GoTo(Addr),
    Call(Addr),
    IfEq(Vx, Val),
    IfEqRg(Vx, Vx),
    IfNeq(Vx, Val),
    IfNeqRg(Vx, Vx),
    Set(Vx, Val),
    SetRg(Vx, Vx),
    Add(Vx, Val),
    AddRg(Vx, Vx),
    Or(Vx, Vx),
    And(Vx, Vx),
    Xor(Vx, Vx),
    Sub(Vx, Vx),
    SubSelf(Vx, Vx), // Todo: find better name
    RightShift(Vx),
    LeftShift(Vx),
    SetAddr(Addr),
    Jump(Addr),
    Rand(Vx, Val),
    Disp(Vx, Vx, Val),
    KeyOpEq(Vx),
    KeyOpNeq(Vx),
    GetTimer(Vx),
    SetTimer(Vx),
    SetSoundTimer(Vx),
    GetKeyOp(Vx),
    AddToI(Vx), // Todo: find better name
    FontLoad(Vx),
    BCD(Vx),
    MemDump(Vx),
    MemLoad(Vx),
}

impl Instruction {
    pub fn from(instr: u16) -> Result<Self> {
        use Instruction::*;
        match instr & 0xF000 {
            0x0000 => Ok(match instr & 0x00FF {
                0x00E0 => DisplayClear,
                0x00EE => Return,
                _ => Raw(instr),
            }),
            0x1000 => Ok(GoTo(as_addr(instr))),
            0x2000 => Ok(Call(as_addr(instr))),
            0x3000 => Ok(IfEq(as_vx(instr), as_val(instr))),
            0x4000 => Ok(IfNeq(as_vx(instr), as_val(instr))),
            0x5000 => Ok(IfEqRg(as_vx(instr), as_vy(instr))),
            0x6000 => Ok(Set(as_vx(instr), as_val(instr))),
            0x7000 => Ok(Add(as_vx(instr), as_val(instr))),
            0x8000 => match instr & 0x000F {
                0 => Ok(SetRg(as_vx(instr), as_vy(instr))),
                1 => Ok(Or(as_vx(instr), as_vy(instr))),
                2 => Ok(And(as_vx(instr), as_vy(instr))),
                3 => Ok(Xor(as_vx(instr), as_vy(instr))),
                4 => Ok(AddRg(as_vx(instr), as_vy(instr))),
                5 => Ok(Sub(as_vx(instr), as_vy(instr))),
                6 => Ok(RightShift(as_vx(instr))),
                7 => Ok(SubSelf(as_vx(instr), as_vy(instr))),
                0xE => Ok(LeftShift(as_vx(instr))),
                _ => Err(ChipoError::UnknownOpCodeErr(instr)),
            },
            0x9000 => Ok(IfNeqRg(as_vx(instr), as_vy(instr))),
            0xA000 => Ok(SetAddr(as_addr(instr))),
            0xB000 => Ok(Jump(as_addr(instr))),
            0xC000 => Ok(Rand(as_vx(instr), as_val(instr))),
            0xD000 => Ok(Disp(as_vx(instr), as_vy(instr), as_small_val(instr))),
            0xE000 => match instr & 0x00FF {
                0x9E => Ok(KeyOpEq(as_vx(instr))),
                0xA1 => Ok(KeyOpNeq(as_vx(instr))),
                _ => Err(ChipoError::UnknownOpCodeErr(instr)),
            },
            0xF000 => match instr & 0x00FF {
                0x07 => Ok(GetTimer(as_vx(instr))),
                0x0A => Ok(GetKeyOp(as_vx(instr))),
                0x15 => Ok(SetTimer(as_vx(instr))),
                0x18 => Ok(SetSoundTimer(as_vx(instr))),
                0x1E => Ok(AddToI(as_vx(instr))),
                0x29 => Ok(FontLoad(as_vx(instr))),
                0x33 => Ok(BCD(as_vx(instr))),
                0x55 => Ok(MemDump(as_vx(instr))),
                0x65 => Ok(MemLoad(as_vx(instr))),
                _ => Err(ChipoError::UnknownOpCodeErr(instr)),
            },
            _ => Err(ChipoError::UnknownOpCodeErr(instr)),
        }
    }

    pub fn to_bin(&self) -> u16 {
        use Instruction::*;
        match self {
            Raw(val) => *val,
            CallPrg(addr) => *addr as u16,
            DisplayClear => 0x00E0,
            Return => 0x00EE,
            GoTo(addr) => ((0x1 << 12) + *addr as u16),
            Call(addr) => ((0x2 << 12) + *addr as u16),
            IfEq(vx, byte) => ((0x3 << 12) + ((*vx & 0xF) << 8) + *byte as usize) as u16,
            IfNeq(vx, byte) => ((0x4 << 12) + ((*vx & 0xF) << 8) + *byte as usize) as u16,
            IfEqRg(vx, vy) => ((0x5 << 12) + ((*vx & 0xF) << 8) + ((*vy & 0xF) << 4)) as u16,
            Set(vx, byte) => ((0x6 << 12) + ((*vx & 0xF) << 8) as u16 + *byte as u16) as u16,
            Add(vx, val) => ((0x7 << 12) + ((*vx & 0xF) << 8) as u16 + *val as u16),
            SetRg(vx, vy) => ((0x8 << 12) + ((*vx & 0xF) << 8) + ((*vy & 0xF) << 4)) as u16,
            Or(vx, vy) => ((0x8 << 12) + ((*vx & 0xF) << 8) + ((*vy & 0xF) << 4) + 1) as u16,
            And(vx, vy) => ((0x8 << 12) + ((*vx & 0xF) << 8) + ((*vy & 0xF) << 4) + 2) as u16,
            Xor(vx, vy) => ((0x8 << 12) + ((*vx & 0xF) << 8) + ((*vy & 0xF) << 4) + 3) as u16,
            AddRg(vx, vy) => ((0x8 << 12) + ((*vx & 0xF) << 8) + ((*vy & 0xF) << 4) + 4) as u16,
            Sub(vx, vy) => ((0x8 << 12) + ((*vx & 0xF) << 8) + ((*vy & 0xF) << 4) + 5) as u16,
            RightShift(vx) => ((0x8 << 12) + ((*vx & 0xF) << 8) + 6) as u16,
            SubSelf(vx, vy) => ((0x8 << 12) + ((*vx & 0xF) << 8) + ((*vy & 0xF) << 4) + 7) as u16,
            LeftShift(vx) => ((0x8 << 12) + ((*vx & 0xF) << 8) + 0xE) as u16,
            IfNeqRg(vx, vy) => ((0x9 << 12) + ((*vx & 0xF) << 8) + ((*vy & 0xF) << 4)) as u16,
            SetAddr(addr) => ((0xA << 12) + *addr as u16),
            Jump(addr) => ((0xB << 12) + *addr as u16),
            Rand(vx, byte) => ((0xB << 12) + ((*vx & 0xF) << 8) + *byte as usize) as u16,
            Disp(vx, vy, nibble) => {
                (0xD << 12)
                    + ((*vx & 0xF) << 8) as u16
                    + (((*vy & 0xF) << 4) as u16 + *nibble as u16)
            }
            KeyOpEq(vx) => ((0xE << 12) + ((*vx & 0xF) << 8) + 0x9E) as u16,
            KeyOpNeq(vx) => ((0xE << 12) + ((*vx & 0xF) << 8) + 0xA1) as u16,
            GetTimer(vx) => ((0xF << 12) + ((*vx & 0xF) << 8) + 0x07) as u16,
            GetKeyOp(vx) => ((0xF << 12) + ((*vx & 0xF) << 8) + 0x0A) as u16,
            SetTimer(vx) => ((0xF << 12) + ((*vx & 0xF) << 8) + 0x15) as u16,
            SetSoundTimer(vx) => ((0xF << 12) + ((*vx & 0xF) << 8) + 0x18) as u16,
            AddToI(vx) => ((0xF << 12) + ((*vx & 0xF) << 8) as u16 + 0x1E),
            FontLoad(vx) => ((0xF << 12) + ((*vx & 0xF) << 8) as u16 + 0x29),
            BCD(vx) => ((0xF << 12) + ((*vx & 0xF) << 8) as u16 + 0x33),
            MemDump(vx) => ((0xF << 12) + ((*vx & 0xF) << 8) as u16 + 0x55),
            MemLoad(vx) => ((0xF << 12) + ((*vx & 0xF) << 8) as u16 + 0x65),
        }
    }

    pub fn to_asm(&self) -> String {
        use Instruction::*;
        match self {
            Raw(val) => format!("raw 0x{:04X}", *val),
            DisplayClear => "cls".to_string(),
            Return => "ret".to_string(),
            GoTo(addr) => format!("jp 0x{:03X}", addr),
            Call(addr) => format!("call 0x{:03X}", addr),
            IfEq(vx, byte) => format!("se v{:X}, 0x{:02X}", vx, byte),
            IfNeq(vx, byte) => format!("sne v{:X}, 0x{:02X}", vx, byte),
            IfEqRg(vx, vy) => format!("se v{:X}, v{:X}", vx, vy),
            Set(vx, byte) => format!("ld v{:X}, 0x{:02X}", vx, byte),
            Add(vx, val) => format!("add v{:X}, 0x{:02X}", vx, val),
            SetRg(vx, vy) => format!("ld v{:X}, v{:X}", vx, vy),
            Or(vx, vy) => format!("or v{:X}, 0x{:02X}", vx, vy),
            And(vx, vy) => format!("and v{:X}, v{:X}", vx, vy),
            Xor(vx, vy) => format!("xor v{:X}, v{:X}", vx, vy),
            AddRg(vx, vy) => format!("add v{:X}, v{:X}", vx, vy),
            Sub(vx, vy) => format!("sub v{:X}, v{:X}", vx, vy),
            RightShift(vx) => format!("shr v{:X}", vx),
            SubSelf(vx, vy) => format!("subn v{:X}, v{:X}", vx, vy),
            LeftShift(vx) => format!("shl v{:X}", vx),
            IfNeqRg(vx, vy) => format!("sne v{:X}, v{:X}", vx, vy),
            SetAddr(addr) => format!("ld i, 0x{:02X}", addr),
            Jump(addr) => format!("jp 0x{:2X}", addr),
            Rand(vx, byte) => format!("rnd v{:X}, 0x{:02X}", vx, byte),
            Disp(vx, vy, nibble) => format!("drw v{:X}, v{:X}, 0x{:02X}", vx, vy, nibble),
            KeyOpEq(vx) => format!("skp v{:X}", vx),
            KeyOpNeq(vx) => format!("sknp v{:X}", vx),
            GetTimer(vx) => format!("ld v{:X}, dt", vx),
            GetKeyOp(vx) => format!("ld v{:X}, k", vx),
            SetTimer(vx) => format!("ld dt, v{:X}", vx),
            SetSoundTimer(vx) => format!("ld st, v{:X}", vx),
            AddToI(vx) => format!("add i, v{:X}", vx),
            FontLoad(vx) => format!("ld f, v{:X}", vx),
            BCD(vx) => format!("ld b, v{:X}", vx),
            MemDump(vx) => format!("ld [i], v{:X}", vx),
            MemLoad(vx) => format!("ld v{:X}, [i]", vx),
            _ => format!("unimplemented {:?}.to_asm()", self),
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_asm())
    }
}

fn as_addr(instr: u16) -> Addr {
    (instr & 0x0FFF) as Addr
}

fn as_val(instr: u16) -> Val {
    (instr & 0x00FF) as Val
}

fn as_small_val(instr: u16) -> Val {
    (instr & 0x000F) as Val
}

fn as_vx(instr: u16) -> Vx {
    (instr & 0x0F00) as Vx >> 8
}

fn as_vy(instr: u16) -> Vx {
    (instr & 0x00F0) as Vx >> 4
}

#[cfg(test)]
mod tests {
    use crate::emu::Instruction;

    #[test]
    fn test_from_bin() {
        assert_eq!(
            Instruction::from(0x00E0).unwrap(),
            Instruction::DisplayClear
        );
    }

    #[test]
    fn test_from_to_bin() {
        // CLS
        assert_eq!(Instruction::from(0x00E0).unwrap().to_bin(), 0x00E0);
        // SET V1 0x00
        assert_eq!(Instruction::from(0x6100).unwrap().to_bin(), 0x6100);
        // DRW V0 V0 0
        assert_eq!(Instruction::from(0xD000).unwrap().to_bin(), 0xD000);
        // JMP addr
        assert_eq!(Instruction::from(0x1999).unwrap().to_bin(), 0x1999);
        // JMP V0 addr
        assert_eq!(Instruction::from(0xB999).unwrap().to_bin(), 0xB999);
        // LD I 0xFFF
        assert_eq!(Instruction::from(0xAFFF).unwrap().to_bin(), 0xAFFF);
    }

    #[test]
    fn test_to_asm() {
        assert_eq!(Instruction::Call(0x200).to_asm(), "call 0x200");
        assert_eq!(Instruction::Set(1, 0x30).to_asm(), "ld v1, 0x30");
        assert_eq!(Instruction::DisplayClear.to_asm(), "cls");
    }
}
