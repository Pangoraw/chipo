pub type Addr = u32;
pub type Vx = usize;
pub type Val = u8;

#[derive(Debug)]
pub enum Instruction {
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
    pub fn from(instr: u16) -> Option<Self> {
        use Instruction::*;
        let instruction = match instr & 0xF000 {
            0x0000 => Some(match instr & 0x00FF {
                0x00E0 => DisplayClear,
                0x00EE => Return,
                _ => CallPrg(as_addr(instr)),
            }),
            0x1000 => Some(GoTo(as_addr(instr))),
            0x2000 => Some(Call(as_addr(instr))),
            0x3000 => Some(IfEq(as_vx(instr), as_val(instr))),
            0x4000 => Some(IfNeq(as_vx(instr), as_val(instr))),
            0x5000 => Some(IfEqRg(as_vx(instr), as_vy(instr))),
            0x6000 => Some(Set(as_vx(instr), as_val(instr))),
            0x7000 => Some(Add(as_vx(instr), as_val(instr))),
            0x8000 => match instr & 0x0001 {
                0 => Some(SetRg(as_vx(instr), as_vy(instr))),
                1 => Some(Or(as_vx(instr), as_vy(instr))),
                2 => Some(And(as_vx(instr), as_vy(instr))),
                3 => Some(Xor(as_vx(instr), as_vy(instr))),
                4 => Some(AddRg(as_vx(instr), as_vy(instr))),
                5 => Some(Sub(as_vx(instr), as_vy(instr))),
                6 => Some(RightShift(as_vx(instr))),
                7 => Some(SubSelf(as_vx(instr), as_vy(instr))),
                0xE => Some(LeftShift(as_vx(instr))),
                _ => None,
            },
            0x9000 => Some(IfNeqRg(as_vx(instr), as_vy(instr))),
            0xA000 => Some(SetAddr(as_addr(instr))),
            0xB000 => Some(Jump(as_addr(instr))),
            0xC000 => Some(Rand(as_vx(instr), as_val(instr))),
            0xD000 => Some(Disp(as_vx(instr), as_vy(instr), as_small_val(instr))),
            0xE000 => match instr & 0x00FF {
                0x9E => Some(KeyOpEq(as_vx(instr))),
                0xA1 => Some(KeyOpNeq(as_vx(instr))),
                _ => None,
            },
            0xF000 => match instr & 0x00FF {
                0x07 => Some(GetTimer(as_vx(instr))),
                0x0A => Some(GetKeyOp(as_vx(instr))),
                0x15 => Some(SetTimer(as_vx(instr))),
                0x18 => Some(SetSoundTimer(as_vx(instr))),
                0x1E => Some(AddToI(as_vx(instr))),
                0x29 => Some(FontLoad(as_vx(instr))),
                0x33 => Some(BCD(as_vx(instr))),
                0x55 => Some(MemDump(as_vx(instr))),
                0x65 => Some(MemLoad(as_vx(instr))),
                _ => None,
            },
            _ => None,
        };
        if let None = instruction {
            println!("OpCode 0x{:04X} not known", instr);
        }
        instruction
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
