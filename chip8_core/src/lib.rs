use rand::Rng;
use std::fs::OpenOptions;
use std::io::Write;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const KEYPAD_SIZE: usize = 4;
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

const KEYPAD: [u8; KEYPAD_SIZE * KEYPAD_SIZE] = [
    0x1, 0x2, 0x3, 0xC, 0x4, 0x5, 0x6, 0xD, 0x7, 0x8, 0x9, 0xE, 0xA, 0x0, 0xB, 0xF,
];

pub struct Emulator {
    pub pc: u16,
    ram: [u8; RAM_SIZE],
    v: [u8; NUM_REGISTERS],
    I: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; STACK_SIZE],
    stack_pointer: u16,
    pub keypad: [bool; KEYPAD_SIZE * KEYPAD_SIZE],
    pub screen: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    pub verbose: bool,
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
            stack_pointer: 0,
            keypad: [false; KEYPAD_SIZE * KEYPAD_SIZE],
            screen: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            verbose: false,
        };
        emulator.ram[FONT_START..FONT_START + FONT_SIZE].copy_from_slice(&FONT);

        emulator
    }

    fn push(&mut self, curr_addr: u16) {
        self.stack[self.stack_pointer as usize] = curr_addr;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    //Loading the opcodes to RAM
    pub fn load_to_ram(&mut self, opcodes: &[u8]) {
        let end_address = START_ADDRESS + opcodes.len();
        if end_address > self.ram.len() {
            panic!("RAM size exceeded");
        }
        self.ram[START_ADDRESS..end_address].copy_from_slice(opcodes);
    }

    fn fetch(&mut self) -> u16 {
        let op: u16 =
            ((self.ram[self.pc as usize] as u16) << 8) | (self.ram[self.pc as usize + 1] as u16);

        op
    }

    fn execute(&mut self, op: u16) {
        let d1: u8 = ((op & 0xF000) >> 12) as u8;
        let d2: u8 = ((op & 0x0F00) >> 8) as u8;
        let d3: u8 = ((op & 0x00F0) >> 4) as u8;
        let d4: u8 = (op & 0x000F) as u8;

        match (d1, d2, d3, d4) {
            //clear screen
            (0, 0, 0xE, 0) => self.screen = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            //jump
            (1, _, _, _) => {
                let addr: u16 = op & 0x0FFF;
                self.pc = addr;
            }
            //set
            (6, x, _, _) => {
                let val = (op & 0x00FF) as u8;
                self.v[x as usize] = val;
            }
            //add
            (7, x, _, _) => {
                let val = (op & 0x00FF) as u8;
                (self.v[x as usize], _ )= self.v[x as usize].overflowing_add(val);
            }
            //set index
            (0xA, _, _, _) => {
                let addr = op & 0x0FFF;
                self.I = addr;
            }
            //display
            (0xD, x, y, n) => {
                let x = self.v[x as usize] as usize;
                let y = self.v[y as usize] as usize;
                let height = n as usize;
                self.v[0xF] = 0;
                for yline in 0..height {
                    let pixel = self.ram[self.I as usize + yline];
                    for xline in 0..8 {
                        if (pixel & (0x80 >> xline)) != 0 {
                            if self.screen[(y + yline) % SCREEN_HEIGHT][(x + xline) % SCREEN_WIDTH]
                            {
                                self.v[0xF] = 1;
                            }
                            self.screen[(y + yline) % SCREEN_HEIGHT][(x + xline) % SCREEN_WIDTH] ^=
                                true;
                        }
                    }
                }
            }
            //call
            (2, _, _, _) => {
                self.push(self.pc);
                let subr_addr = op & 0x0FFF;
                self.pc = subr_addr;
            }
            //ret
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }
            //skip if VX==NN
            (3, x, _, _) => {
                let val = (op & 0x00FF) as u8;
                if self.v[x as usize] == val {
                    self.pc += 2;
                }
            }
            //skip if VX!=NN
            (4, x, _, _) => {
                let val = (op & 0x00FF) as u8;
                if self.v[x as usize] != val {
                    self.pc += 2;
                }
            }
            //skip if VX=VY
            (5, x, y, 0) => {
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 2;
                }
            }
            //skip if VX!=VY
            (9, x, y, 0) => {
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc += 2;
                }
            }
            //set VX to VY
            (8, x, y, 0) => {
                self.v[x as usize] = self.v[y as usize];
            }
            //set VX to (VX OR VY)
            (8, x, y, 1) => {
                self.v[x as usize] |= self.v[y as usize];
            }
            //set VX to (VX AND VY)
            (8, x, y, 2) => {
                self.v[x as usize] &= self.v[y as usize];
            }
            //set VX to (VX XOR VY)
            (8, x, y, 3) => {
                self.v[x as usize] ^= self.v[y as usize];
            }
            //set VX to (VX+VY) (may overflow)
            (8, x, y, 4) => {
                let (sum, carry) = self.v[x as usize].overflowing_add(self.v[y as usize]);
                let flag = if carry { 1 } else { 0 };
                self.v[0xF] = flag;
                self.v[x as usize] = sum;
            }
            //set VX to (VX-VY) (may underflow)
            (8, x, y, 5) => {
                let (diff, borrow) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
                let flag = if borrow { 0 } else { 1 };
                self.v[0xF] = flag;
                self.v[x as usize] = diff;
            }
            //set VX to (VY-VX) (may underflow)
            (8, x, y, 7) => {
                let (diff, borrow) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
                let flag = if borrow { 0 } else { 1 };
                self.v[0xF] = flag;
                self.v[x as usize] = diff;
            }
            //shift VX one bit right
            (8, x, y, 6) => {
                let bit = self.v[x as usize] & 1;
                self.v[0xF] = bit;
                self.v[x as usize] >>= 1;
            }
            //shift VX one bit left
            //NOTE: it is optional to put the value of VY into VX first
            (8, x, y, 0xE) => {
                // self.v[x as usize]=self.v[y as usize];
                let bit = (self.v[x as usize] >> 7) & 1;
                self.v[0xF] = bit;
                self.v[x as usize] <<= 1;
            }
            //jump with offset
            //NOTE: the behaviour
            (0xB, _, _, _) => {
                let addr = op & 0x0FFF;
                self.pc = addr + self.v[0] as u16;
            }
            //set VX to (random_binary_no AND NN)
            (0xC, x, _, _) => {
                let val = (op & 0x00FF) as u8;
                self.v[x as usize] = rand::thread_rng().gen_range(0..=val) & val;
            }
            //add to index
            (0xF, x, 1, 0xE) => {
                self.I += self.I.wrapping_add(self.v[x as usize] as u16);
            }
            //font character
            (0xF, x, 2, 9) => {
                let char = self.v[x as usize] as u16;
                self.I = FONT_START as u16 + char * 5;
            }
            //binary coded decimal conversion
            (0xF, x, 3, 3) => {
                let val = self.v[x as usize];
                self.ram[self.I as usize] = val / 100;
                self.ram[(self.I + 1) as usize] = (val / 10) % 10;
                self.ram[(self.I + 2) as usize] = val % 10;
            }
            //store register values into ram
            (0xF, x, 5, 5) => {
                let start_addr = self.I as usize;
                for reg in 0..x as usize {
                    self.ram[start_addr + reg] = self.v[reg];
                }
            }
            //load values from ram to registers
            (0xF, x, 6, 5) => {
                let start_addr = self.I as usize;
                for reg in 0..x as usize {
                    self.v[reg] = self.ram[start_addr + reg];
                }
            }
            // Get key
            (0xF, x, 0, 0xA) => {
                let mut key_pressed = false;
                for i in 0..KEYPAD_SIZE * KEYPAD_SIZE {
                    if self.keypad[i] {
                        key_pressed = true;
                        self.v[x as usize] = KEYPAD[i] as u8;
                        break;
                    }
                }
                if !key_pressed {
                    self.pc -= 2;
                }
            }
            // Skip if key
            (0xE, x, 9, 0xE) => {
                let key = self.v[x as usize] as usize;
                for i in 0..KEYPAD_SIZE * KEYPAD_SIZE {
                    if self.keypad[i] && KEYPAD[i] as usize == key {
                        self.pc += 2;
                        break;
                    }
                }
            }
            (0xE, x, 0xA, 1) => {
                let key = self.v[x as usize] as usize;
                for i in 0..KEYPAD_SIZE * KEYPAD_SIZE {
                    if self.keypad[i] && KEYPAD[i] as usize != key {
                        self.pc += 2;
                        break;
                    }
                }
            }
            //else
            (_, _, _, _) => {
                if self.verbose {
                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open("log.txt")
                        .unwrap();
                    writeln!(file, "{}", op).unwrap();
                }
            }
        }
    }

    pub fn tick(&mut self) {
        let op: u16 = self.fetch();
        self.pc += 2;
        if self.verbose{
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open("log.txt")
                .unwrap();
            writeln!(file, "{:04X}", op).unwrap();
        }
        self.execute(op);
    }
}
