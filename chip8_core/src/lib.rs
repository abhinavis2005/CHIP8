pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const STACK_SIZE: usize = 16;

const RAM_SIZE: usize = 4096;
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