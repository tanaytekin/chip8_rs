use std::fs::File;
use std::io::Read;
use std::path::Path;

use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;

mod error;
use error::Error;
use error::Result;

const SPRITES: &'static [u8] = &[
    /*0*/ 0xF0, 0x90, 0x90, 0x90, 0xF0,
    /*1*/ 0x20, 0x60, 0x20, 0x20, 0x70,
    /*2*/ 0xF0, 0x10, 0xF0, 0x80, 0xF0,
    /*3*/ 0xF0, 0x10, 0xF0, 0x10, 0xF0,
    /*4*/ 0x90, 0x90, 0xF0, 0x10, 0x10,
    /*5*/ 0xF0, 0x80, 0xF0, 0x10, 0xF0,
    /*6*/ 0xF0, 0x80, 0xF0, 0x90, 0xF0,
    /*7*/ 0xF0, 0x10, 0x20, 0x40, 0x40,
    /*8*/ 0xF0, 0x90, 0xF0, 0x90, 0xF0,
    /*9*/ 0xF0, 0x90, 0xF0, 0x10, 0xF0,
    /*A*/ 0xF0, 0x90, 0xF0, 0x90, 0x90,
    /*B*/ 0xE0, 0x90, 0xE0, 0x90, 0xE0,
    /*C*/ 0xF0, 0x80, 0x80, 0x80, 0xF0,
    /*D*/ 0xE0, 0x90, 0x90, 0x90, 0xE0,
    /*E*/ 0xF0, 0x80, 0xF0, 0x80, 0xF0,
    /*F*/ 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

#[allow(non_snake_case)]
pub struct Chip8 {
    memory: [u8; 0x1000],
    V: [u8; 0x10],
    stack: [u16; 0x10],
    display: [u8; 8 * 4],
    keys: [bool; 16],
    I: u16,
    pc: u16,
    sp: u8,
    DT: u8,
    ST: u8,
    rng: ThreadRng,
    rand_dist: Uniform<u8>,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut memory = [0; 0x1000];
        memory[..SPRITES.len()].clone_from_slice(&SPRITES);

        Chip8 {
            memory,
            V: [0; 0x10],
            stack: [0; 0x10],
            display: [0; 8 * 4],
            keys: [false; 16],
            I: 0,
            pc: 0,
            sp: 0,
            DT: 0,
            ST: 0,
            rng: rand::thread_rng(),
            rand_dist: Uniform::from(0..0xFF),
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let mut file = File::open(path)?;
        let romsize = file.metadata()?.len();
        if romsize > (0xFFF - 0x200) {
            return Err(Error::ROMIsTooBig(romsize));
        }
        file.read_exact(&mut self.memory[0x200..romsize as usize])?;
        self.pc = 0x200;
        Ok(())
    }

    pub fn cycle(&mut self) {
        let opcode: u16 = ((self.memory[self.pc as usize] as u16) << 8)
            | self.memory[(self.pc + 1) as usize] as u16;
        self.pc += 2;

        let o = (opcode & 0xF000) >> 12;
        let nnn = opcode & 0x0FFF;
        let n = opcode & 0x000F;
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let kk = (opcode & 0x00FF) as u8;


        macro_rules! V {
            ($offset:expr) => {
                self.V[$offset as usize]
            }
        }

        macro_rules! Vx {
            () => {
                self.V[x as usize]
            }
        }

        macro_rules! Vy {
            () => {
                self.V[y as usize]
            }
        }

        println!("{:#02x}", opcode);

        match (o, kk, n) {
            // 0x00E0 - CLS
            (0, 0xE0, _) => self.display.fill(0),
            // 0x00EE - RET
            (0, 0xEE, _) => {
                self.pc = self.stack[self.sp as usize];
                self.sp -= 1;
            }
            // 0x1nnn - JP addr
            (1, _, _) => self.pc = nnn,
            // 0x2nnn - CALL addr
            (2, _, _) => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = nnn;
            }
            // 3xkk - SE Vx, byte
            (3, _, _) => {
                if Vx!() == kk {
                    self.pc += 2;
                }
            }
            // 4xkk - SNE Vx, byte
            (4, _, _) => {
                if Vx!() != kk {
                    self.pc += 2;
                }
            }
            // 5xy0 - SE Vx, Vy
            (5, _, 0) => {
                if Vx!() == Vy!() {
                    self.pc += 2;
                }
            }
            // 6xkk - LD Vx, byte
            (6, _, _) => Vx!() = kk,
            // 7xkk - ADD Vx, byte
            (7, _, _) => Vx!() += kk,
            // 8xy0 - LD Vx, Vy
            (8, _, 0) => Vx!() = Vy!(),
            // 8xy1 - OR Vx, Vy
            (8, _, 1) => Vx!() |= Vy!(),
            // 8xy2 - AND Vx, Vy
            (8, _, 2) => Vx!() &= Vy!(),
            // 8xy3 - XOR Vx, Vy
            (8, _, 3) => Vx!() ^= Vy!(),
            // 8xy4 - ADD Vx, Vy
            (8, _, 4) => {
                let sum = Vx!() as u16 + Vy!() as u16;
                if sum > 0x10 {
                    V!(0xF) = 1;
                } else {
                    V!(0xF) = 0;
                }
                Vx!() = (sum & 0xFF) as u8;
            }
            // 8xy5 - SUB Vx, Vy
            (8, _, 5) => {
                if Vx!() > Vy!() {
                    V!(0xF) = 1;
                } else {
                    V!(0xF) = 0;
                }
                Vx!() -= Vy!()
            }
            // 8xy6 - SHR Vx {, Vy}
            (8, _, 6) => {
                V!(0xF) = Vx!() & 1;
                Vx!() >>= 1;
            }
            // 8xy7 - SUBN Vx, Vy
            (8, _, 7) => {
                if Vy!() > Vx!() {
                    V!(0xF) = 1;
                } else {
                    V!(0xF) = 0;
                }
                Vx!() -= Vy!();
            }
            // 8xyE - SHL Vx {, Vy}
            (8, _, 0xE) => {
                V!(0xF) = Vx!() >> 7;
                Vx!() <<= 1;
            }
            // 9xy0 - SNE Vx, Vy
            (9, _, 0) => {
                if Vx!() != Vy!() {
                    self.pc += 2;
                }
            }
            // Annn - LD I, addr
            (0xA, _, _) => self.I = nnn,
            // Bnnn - JP V0, addr
            (0xB, _, _) => self.pc = nnn + V!(0) as u16,
            // Cxkk - RND Vx, byte
            (0xC, _, _) => {
                let random = self.rand_dist.sample(&mut self.rng);
                Vx!() = random & kk;
            }
            // Dxyn - DRW Vx, Vy, nibble
            (0xD, _, _) => {
                //  todo!()
            }
            // Ex9E - SKP Vx
            (0xE, 0x9E, _) => {
                if self.keys[Vx!() as usize] {
                    self.pc += 2;
                }
            }
            // ExA1 - SKNP Vx
            (0xE, 0xA1, _) => {
                if !self.keys[Vx!() as usize] {
                    self.pc += 2;
                }
            }
            // Fx07 - LD Vx, DT
            (0xF, 0x07, _) => Vx!() = self.DT,
            // Fx0A - LD Vx, K
            (0xF, 0x0A, _) => {
                self.pc -= 2;
                for (i, key) in self.keys.iter().enumerate() {
                    if *key {
                        Vx!() = i as u8;
                        self.pc += 2;
                        break;
                    }
                }
            }
            // Fx15 - LD DT, Vx
            (0xF, 0x15, _) => self.DT = Vx!(),
            // Fx18 - LD ST, Vx
            (0xF, 0x18, _) => self.ST = Vx!(),
            // Fx1E - ADD I, Vx
            (0xF, 0x1E, _) => self.I += Vx!() as u16,
            // Fx29 - LD F, Vx
            (0xF, 0x29, _) => self.I = Vx!() as u16 * 5,
            // Fx33 - LD B, Vx
            (0xF, 0x33, _) => {
                self.memory[self.I as usize] = (Vx!() / 100) % 10;
                self.memory[self.I as usize + 1] = (Vx!() / 10) % 10;
                self.memory[self.I as usize + 2] = Vx!() % 10;
            }
            // Fx55 - LD [I], Vx
            (0xF, 0x55, _) => {
                for offset in 0..x as usize {
                    self.memory[self.I as usize + offset] = self.V[offset];
                }
            }
            // Fx65 - LD Vx, [I]
            (0xF, 0x65, _) => {
                for offset in 0..x as usize {
                    self.V[offset] = self.memory[self.I as usize + offset];
                }
            }

            _ => unimplemented!(),
        }
    }
}
