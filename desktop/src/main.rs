use chip8_core::Emulator;
use std::fs;
use std::process;

fn main() {
    // Receiving the ROM file from argument
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run [full_path_to_game_file]");
        process::exit(1);
    }
    let buffer  = fs::read(&args[1]).expect("Error reading the file");

    // Emulator Initialisation and loading ROM into RAM
    let mut chip8 = Emulator::new();
    chip8.load_to_ram(&buffer);
}
