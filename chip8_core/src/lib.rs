pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const STACK_SIZE: usize = 16;

const RAM_SIZE: usize = 4096;
const START_ADDRESS: usize = 0x200;
const NUM_REGISTERS: usize = 16;
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
        Emulator {
            pc: 0,
            ram: [0; RAM_SIZE],
            v: [0; NUM_REGISTERS],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; STACK_SIZE],
        }
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
