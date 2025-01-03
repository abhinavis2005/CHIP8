use std::arch::x86_64;
use std::process;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const STACK_SIZE: usize = 16;

const RAM_SIZE: usize = 4096;
const START_ADDRESS: usize = 0x200;
const NUM_REGISTERS: usize = 16;

const FONT_SIZE: usize = 80;
const FONT_START: usize = 0x50;

const FONT: [u8; FONT_SIZE] = [
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

pub struct Emulator {
    pub pc: u16,
    ram: [u8; RAM_SIZE],
    v: [u8; NUM_REGISTERS],
    I: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; STACK_SIZE],
    pub screen: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
}

impl Emulator {
    pub fn new() -> Self {
        let mut emulator = Self {
            pc: 0x200,
            ram: [0; RAM_SIZE],
            v: [0; NUM_REGISTERS],
            I: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; STACK_SIZE],
            screen: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
        };
        emulator.ram[FONT_START..FONT_START + FONT_SIZE].copy_from_slice(&FONT);

        emulator
    }

    //Loading the opcodes to RAM
    pub fn load_to_ram(&mut self, opcodes: &[u8]) {
        let end_address = START_ADDRESS + opcodes.len();
        if end_address > self.ram.len() {
            panic!("RAM size exceeded");
        }
        self.ram[START_ADDRESS..end_address].copy_from_slice(opcodes);
    }
    
    fn fetch(&mut self) -> u16{
        let op: u16= ((self.ram[self.pc as usize] as u16) << 8 )| (self.ram[self.pc as usize + 1] as u16);  
        
        op
    }
    
    fn execute(&mut self, op: u16){
        let d1: u8 = ((op & 0xF000) >> 12) as u8;
        let d2: u8 = ((op & 0x0F00) >> 8) as u8;
        let d3: u8 = ((op & 0x00F0) >> 4) as u8;
        let d4: u8 = (op & 0x000F) as u8;

        match (d1, d2, d3, d4) {
            (0, 0, 0xE, 0) => self.screen = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            (1, _, _, _) => {
                let addr: u16 = op & 0x0FFF;
                self.pc = addr;
            }
            (6, x, _, _) => {
                let val = (op & 0x00FF) as u8;
                self.v[x as usize] = val;
            }
            (7, x, _, _) => {
                let val = (op & 0x00FF) as u8;
                self.v[x as usize] += val;
            }
            (0xA, _, _, _) => {
                let addr = op & 0x0FFF;
                self.I = addr;
            }
            (0xD, x, y, n) => {
                let x = self.v[x as usize] as usize;
                let y = self.v[y as usize] as usize;
                let height = n as usize;
                self.v[0xF] = 0;
                for yline in 0..height {
                    let pixel = self.ram[self.I as usize + yline];
                    for xline in 0..8 {
                        if (pixel & (0x80 >> xline)) != 0 {
                            if self.screen[(y + yline) % SCREEN_HEIGHT][(x + xline) % SCREEN_WIDTH] {
                                self.v[0xF] = 1;
                            }
                            self.screen[(y + yline) % SCREEN_HEIGHT][(x + xline) % SCREEN_WIDTH] ^= true;
                        }
                    }
                } 
                
            }
            (_, _, _, _) => {
                process::exit(1);
            },
        }
    }

    pub fn tick(&mut self){
        let op: u16 = self.fetch();
        self.pc+=2;
        
        self.execute(op);

    }
}
