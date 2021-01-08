use rand::Rng;

use crate::media::screen::SCALE;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

use crate::emu::{Addr, Instruction, Instruction::*, Val};
use crate::error::{ChipoError, Result};

pub struct Proc {
    memory: [Val; 4096],
    rg: [Val; 16],
    i: Addr,
    delay_rg: Val,
    sound_rg: Val,
    pc: usize,
    stack: Vec<Addr>,
    pub should_render: bool,
    pixels: [Val; 64 * 32],
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
            pixels: [0x00; 64 * 32],
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
                    *pixel = 0x00;
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
                self.pc = addr as usize + 2;
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
                self.rg[vx] = self.rg[vx] | self.rg[vy];
                self.pc += 2;
            }
            And(vx, vy) => {
                self.rg[vx] = self.rg[vx] & self.rg[vy];
                self.pc += 2;
            }
            Xor(vx, vy) => {
                self.rg[vx] = self.rg[vx] ^ self.rg[vy];
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
                    let mut spr = self.memory[self.i as usize + y as usize];
                    for x in 0..8 {
                        if self.set_pixel(
                            (self.rg[vx].wrapping_add(x)) as usize,
                            (self.rg[vy].wrapping_add(y)) as usize,
                            (spr & 0x80) >> 7,
                        ) {
                            self.rg[0xF] = 0x1;
                        }
                        spr <<= 1;
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
                for reg in 0..vx {
                    self.memory[self.i as usize + reg] = self.rg[reg];
                }
                self.pc += 2;
            }
            MemLoad(vx) => {
                for reg in 0..vx {
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

    pub fn set_pixel(&mut self, x: usize, y: usize, val: Val) -> bool {
        self.should_render = true;
        let location = (x % 64) + 64 * (y % 32);
        let collision = self.pixels[location] & val;
        self.pixels[location] ^= val;
        collision == 0x01
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
        match keycode {
            Keycode::Num1 => Some(0),
            Keycode::Num2 => Some(1),
            Keycode::Num3 => Some(2),
            Keycode::Num4 => Some(3),
            Keycode::A => Some(4),
            Keycode::Z => Some(5),
            Keycode::E => Some(6),
            Keycode::R => Some(7),
            Keycode::Q => Some(8),
            Keycode::S => Some(9),
            Keycode::D => Some(10),
            Keycode::F => Some(11),
            Keycode::W => Some(12),
            Keycode::X => Some(13),
            Keycode::C => Some(14),
            Keycode::V => Some(15),
            _ => None,
        }
    }
    pub fn should_buzz(&self) -> bool {
        self.sound_rg > 0
    }
    pub fn to_rects(&mut self) -> Vec<Rect> {
        self.should_render = false;
        self.pixels
            .iter()
            .enumerate()
            .map(|(pos, &b)| {
                if b == 0x01 {
                    Some(Rect::new(
                        (pos % 64) as i32 * SCALE,
                        (pos / 64) as i32 * SCALE,
                        SCALE as u32,
                        SCALE as u32,
                    ))
                } else {
                    None
                }
            })
            .filter(|opt| opt.is_some())
            .flatten()
            .collect::<Vec<Rect>>()
    }
}
