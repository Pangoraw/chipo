use rand::Rng;

use crate::emu::{Addr, Instruction, Instruction::*, Keycode, Val};
use crate::error::{ChipoError, Result};

#[derive(Debug)]
pub struct Proc {
    memory: [Val; 4096],
    rg: [Val; 16],
    i: Addr,
    delay_rg: Val,
    sound_rg: Val,
    pc: usize,
    stack: Vec<Addr>,
    pub should_render: bool,
    pub pixels: [bool; 64 * 32],
    keys: [bool; 16],
}

pub enum ProgramState {
    Continue,
    Stop,
}

impl Proc {
    pub fn binary(blob: &[u8]) -> Result<Self> {
        let fonts = vec![
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        let mut proc = Proc {
            memory: [0; 4096],
            rg: [0; 16],
            i: 0,
            delay_rg: 0,
            sound_rg: 0,
            pc: 0x200,
            stack: vec![],
            should_render: true,
            pixels: [false; 64 * 32],
            keys: [false; 16],
        };
        for (pos, &b) in fonts.iter().enumerate() {
            proc.memory[pos] = b;
        }
        for (pos, &el) in blob.iter().enumerate() {
            proc.memory[0x200 + pos] = el;
        }

        Ok(proc)
    }
    pub fn cycle(&mut self) -> Result<ProgramState> {
        let instr = ((self.memory[self.pc] as u16) << 8) + (self.memory[self.pc + 1] as u16);
        let for_instr = Instruction::from(instr)?;

        match for_instr {
            DisplayClear => {
                self.pixels.iter_mut().for_each(|pixel| {
                    *pixel = false;
                });
                self.pc += 2;
            }
            Return => {
                if let Some(val) = self.stack.pop() {
                    self.pc = val as usize + 2;
                } else {
                    return Ok(ProgramState::Stop); // Stack is empty returned from main program
                }
            }
            Call(addr) => {
                self.stack.push(self.pc as Addr);
                self.pc = addr as usize;
            }
            GoTo(addr) => {
                self.pc = addr as usize;
            }
            IfEq(vx, val) => {
                if self.rg[vx] == val {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            IfEqRg(vx, vy) => {
                if self.rg[vx] == self.rg[vy] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            IfNeq(vx, val) => {
                if self.rg[vx] != val {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            IfNeqRg(vx, vy) => {
                if self.rg[vx] != self.rg[vy] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            Set(vx, val) => {
                self.rg[vx] = val;
                self.pc += 2;
            }
            SetRg(vx, vy) => {
                self.rg[vx] = self.rg[vy];
                self.pc += 2;
            }
            Add(vx, val) => {
                self.rg[vx] = self.rg[vx].wrapping_add(val);
                self.pc += 2;
            }
            AddRg(vx, vy) => {
                let (val, overflow) = self.rg[vx].overflowing_add(self.rg[vy]);
                self.rg[0xF] = if overflow { 1 } else { 0 };
                self.rg[vx] = val;
                self.pc += 2;
            }
            Or(vx, vy) => {
                self.rg[vx] |= self.rg[vy];
                self.pc += 2;
            }
            And(vx, vy) => {
                self.rg[vx] &= self.rg[vy];
                self.pc += 2;
            }
            Xor(vx, vy) => {
                self.rg[vx] ^= self.rg[vy];
                self.pc += 2;
            }
            Sub(vx, vy) => {
                let (val, overflow) = self.rg[vx].overflowing_sub(self.rg[vy]);
                self.rg[0xF] = if overflow { 1 } else { 0 };
                self.rg[vx] = val;
                self.pc += 2;
            }
            SubSelf(vx, vy) => {
                let (val, overflow) = self.rg[vy].overflowing_sub(self.rg[vx]);
                self.rg[0xF] = if overflow { 1 } else { 0 };
                self.rg[vx] = val;
                self.pc += 2;
            }
            RightShift(vx) => {
                self.rg[0xF] = 0x01 & self.rg[vx];
                self.rg[vx] >>= 1;
                self.pc += 2;
            }
            LeftShift(vx) => {
                self.rg[0xF] = 0b1000_0000 & self.rg[vx];
                self.rg[vx] <<= 1;
                self.pc += 2;
            }
            SetAddr(addr) => {
                self.i = addr;
                self.pc += 2;
            }
            Jump(addr) => {
                self.pc = self.rg[0] as usize + addr as usize;
            }
            Rand(vx, val) => {
                let mut rng = rand::thread_rng();
                let result = rng.gen_range(0, 255);
                self.rg[vx] = result & val;
                self.pc += 2;
            }
            Disp(vx, vy, n) => {
                self.rg[0xF] = 0x00;
                for y in 0..n {
                    let spr = self.memory[self.i as usize + y as usize];
                    for x in 0..8 {
                        if self.set_pixel(
                            (self.rg[vx].wrapping_add(x)) as usize,
                            (self.rg[vy].wrapping_add(y)) as usize,
                            (spr >> (7 - x) & 0x01) != 0,
                        ) {
                            self.rg[0xF] = 0x1;
                        }
                    }
                }
                self.pc += 2;
            }
            SetSoundTimer(vx) => {
                self.sound_rg = self.rg[vx];
                self.pc += 2;
            }
            SetTimer(vx) => {
                self.delay_rg = self.rg[vx];
                self.pc += 2;
            }
            GetTimer(vx) => {
                self.rg[vx] = self.delay_rg;
                self.pc += 2;
            }
            BCD(vx) => {
                self.memory[self.i as usize] = self.rg[vx] / 100;
                self.memory[self.i as usize + 1] = (self.rg[vx] / 10) % 10;
                self.memory[self.i as usize + 2] = (self.rg[vx] % 100) % 10;
                self.pc += 2;
            }
            MemDump(vx) => {
                for reg in 0..(vx + 1) {
                    self.memory[self.i as usize + reg] = self.rg[reg];
                }
                self.pc += 2;
            }
            MemLoad(vx) => {
                for reg in 0..(vx + 1) {
                    self.rg[reg] = self.memory[self.i as usize + reg];
                }
                self.pc += 2;
            }
            AddToI(vx) => {
                self.i = self.i.wrapping_add(self.rg[vx] as Addr);
                self.pc += 2;
            }
            FontLoad(vx) => {
                self.i = (self.rg[vx] * 5) as Addr;
                self.pc += 2;
            }
            KeyOpEq(vx) => {
                let key = self.keys.iter().position(|&b| b);
                if Some(self.rg[vx] as usize) == key {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            KeyOpNeq(vx) => {
                let key = self.keys.iter().position(|&b| b);
                if Some(self.rg[vx] as usize) != key {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            GetKeyOp(vx) => {
                let key = self.keys.iter().position(|&b| b);
                if let Some(k) = key {
                    self.rg[vx] = k as Val;
                    self.pc += 2;
                }
            }
            _ => {
                return Err(ChipoError::UnimplementedOpCodeErr(instr, for_instr));
            }
        }

        Ok(ProgramState::Continue)
    }

    pub fn decrement_registers(&mut self) {
        if self.should_buzz() {
            self.sound_rg -= 1;
        }
        if self.delay_rg > 0 {
            self.delay_rg -= 1;
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, val: bool) -> bool {
        self.should_render = true;
        let location = (x % 64) + 64 * (y % 32);
        let collision = self.pixels[location] && val;
        self.pixels[location] ^= val;
        collision
    }
    pub fn set_key_down(&mut self, keycode: Keycode) {
        if let Some(i) = self.get_key_index(keycode) {
            self.keys[i] = true;
        }
    }
    pub fn set_key_up(&mut self, keycode: Keycode) {
        if let Some(i) = self.get_key_index(keycode) {
            self.keys[i] = false;
        }
    }
    fn get_key_index(&self, keycode: Keycode) -> Option<usize> {
        // From http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#keyboard
        match keycode {
            Keycode::Num1 => Some(1),
            Keycode::Num2 => Some(2),
            Keycode::Num3 => Some(3),
            Keycode::Num4 => Some(0xC),
            Keycode::Q => Some(4),
            Keycode::W => Some(5),
            Keycode::E => Some(6),
            Keycode::R => Some(0xD),
            Keycode::A => Some(7),
            Keycode::S => Some(8),
            Keycode::D => Some(9),
            Keycode::F => Some(0xE),
            Keycode::Z => Some(0xA),
            Keycode::X => Some(0),
            Keycode::C => Some(0xB),
            Keycode::V => Some(0xF),
            _ => None,
        }
    }
    pub fn should_buzz(&self) -> bool {
        self.sound_rg > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile;

    fn exec(prg: &str) -> Proc {
        let binary = compile(&format!(".code\n{}\nret", prg)).unwrap();
        let mut proc = Proc::binary(&binary).unwrap();

        while let Ok(ProgramState::Continue) = proc.cycle() {}

        proc
    }

    #[test]
    fn test_add() {
        let proc = exec(
            r#"
    ld v0, 10
    ld v1, 10
    add v0, v1
            "#,
        );

        assert_eq!(proc.rg[0], 20);
    }

    #[test]
    fn test_and() {
        let proc = exec(
            r#"
    ld v0, 0xFF
    ld v1, 0x0F
    and v0, v1
            "#,
        );

        assert_eq!(proc.rg[0], 0x0F);
    }

    #[test]
    fn test_or() {
        let proc = exec(
            r#"
    ld v0, 0xF0
    ld v1, 0x0F
    or v0, v1
            "#,
        );

        assert_eq!(proc.rg[0], 0xFF);
    }

    #[test]
    fn test_set_addr() {
        let proc = exec("ld i, 0xFFF");

        assert_eq!(proc.i, 0xFFF);
    }

    #[test]
    fn test_is_equal() {
        let proc = exec(
            r#"
    ld v0, 1
    ld v1, 1
    se v0, v1
    ld v0, 10
    "#,
        );

        assert_eq!(proc.rg[0], 1);
    }

    #[test]
    fn test_is_not_equal() {
        let proc = exec(
            r#"
    ld v0, 1
    ld v1, 1
    sne v0, v1
    ld v0, 10
    "#,
        );

        assert_eq!(proc.rg[0], 10);
    }

    #[test]
    fn test_skip_is_equal() {
        let proc = exec(
            r#"
    ld v0, 1
    ld v1, 2
    se v0, v1
    ld v0, 10
    "#,
        );

        assert_eq!(proc.rg[0], 10);
    }

    #[test]
    fn test_skip_is_not_equal() {
        let proc = exec(
            r#"
    ld v0, 1
    ld v1, 2
    sne v0, v1
    ld v0, 10
    "#,
        );

        assert_eq!(proc.rg[0], 1);
    }

    #[test]
    fn test_bcd() {
        let proc = exec(
            r#"
    ld v0, 123
    ld b, v0
    ld v2, [i]
    "#,
        );

        assert_eq!(proc.rg[0], 1);
        assert_eq!(proc.rg[1], 2);
        assert_eq!(proc.rg[2], 3);
    }

    #[test]
    fn test_font_load() {
        let proc = exec(
            r#"
    ld v0, 2
    ld f, v0
    "#,
        );

        assert_eq!(proc.i, 10);
    }

    #[test]
    fn test_call() {
        let proc = exec(
            r#"
    call addr
    ret

addr:
    ld v0, 5
    "#,
        );

        assert_eq!(proc.rg[0], 5);
    }

    #[test]
    fn test_jump() {
        let proc = exec(
            r#"
    jp addr
done:
    ret

addr:
    ld v0, 5
    jp done
    "#,
        );

        assert_eq!(proc.rg[0], 5);
    }

    #[test]
    fn test_sub() {
        let proc = exec(
            r#"
    ld v0, 10
    ld v1, 5
    sub v0, v1
    "#,
        );

        assert_eq!(proc.rg[0], 5);

        let proc = exec(
            r#"
    ld v0, 10
    ld v1, 5
    sub v1, v0
    "#,
        );

        assert_eq!(proc.rg[0], 10);
        assert_eq!(proc.rg[1], 251);
        assert_eq!(proc.rg[0xF], 1);
    }
}
