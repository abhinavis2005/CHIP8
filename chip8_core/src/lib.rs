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
    pc: u16,
    ram: [u8; RAM_SIZE],
    v: [u8; NUM_REGISTERS],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; STACK_SIZE],
}

impl Emulator {
    pub fn new() -> Self {
        let mut emulator = Self {
            pc: 0,
            ram: [0; RAM_SIZE],
            v: [0; NUM_REGISTERS],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; STACK_SIZE],
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
}
