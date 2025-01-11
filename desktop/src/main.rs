use chip8_core::Emulator;

use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{self, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
};
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

use rodio::{OutputStream, Sink, Source, source::SineWave};

fn main() -> Result<(), Box<dyn Error>> {
    // Receiving the ROM file from argument
    let mut chip8 = Emulator::new();
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        // eprintln!("Usage: cargo run [full_path_to_game_file]");
        // process::exit(1);
        let path = "/home/abhinav/Documents/CHIP8/desktop/test_opcode.ch8";
        let buffer = fs::read(path).expect("Error reading the file");
        chip8.load_to_ram(&buffer);
    } else {
        let buffer = fs::read(&args[1]).expect("Error reading the file");
        chip8.load_to_ram(&buffer);
    }
    // Emulator Initialisation and loading ROM into RAM

    //enabling verbose mode
    chip8.verbose = true;
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let _ = run_app(&mut terminal, chip8);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;

    terminal.show_cursor()?;

    Ok(())
}

fn ui(frame: &mut Frame, screen: &[[bool; chip8_core::SCREEN_WIDTH]; chip8_core::SCREEN_HEIGHT]) {
    // Define the size of each "pixel" in the terminal
    let pixel_width = 1;
    let pixel_height = 1;

    // Loop through the screen and light up the pixels
    for (y, row) in screen.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            if pixel {
                // Create a block or paragraph for the pixel
                let rect = Rect {
                    x: x as u16 * pixel_width,
                    y: y as u16 * pixel_height,
                    width: pixel_width,
                    height: pixel_height,
                };

                // Use a block or any widget to represent the lit pixels
                frame.render_widget(
                    Paragraph::new("█").style(Style::default().fg(Color::White)), // Adjust color if needed
                    rect,
                );
            }
        }
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut emulator: chip8_core::Emulator,
) -> io::Result<bool> {
    let max_terminal_width = chip8_core::SCREEN_WIDTH as u16;
    let max_terminal_height = chip8_core::SCREEN_HEIGHT as u16;

    let key_mapping: HashMap<KeyCode, usize> = [
        (KeyCode::Char('1'), 0),
        (KeyCode::Char('2'), 1),
        (KeyCode::Char('3'), 2),
        (KeyCode::Char('4'), 3),
        (KeyCode::Char('q'), 4),
        (KeyCode::Char('w'), 5),
        (KeyCode::Char('e'), 6),
        (KeyCode::Char('r'), 7),
        (KeyCode::Char('a'), 8),
        (KeyCode::Char('s'), 9),
        (KeyCode::Char('d'), 10),
        (KeyCode::Char('f'), 11),
        (KeyCode::Char('z'), 12),
        (KeyCode::Char('x'), 13),
        (KeyCode::Char('c'), 14),
        (KeyCode::Char('v'), 15),
    ]
    .into();

    let size = terminal.size()?;
    if size.width < max_terminal_width || size.height < max_terminal_height {
        terminal.draw(|f| {
            f.render_widget(
                Paragraph::new("Terminal too small. Please resize to at least 64x32"),
                f.area(),
            )
        })?;
        loop {
            if let Event::Resize(width, height) = event::read()? {
                if width >= max_terminal_width && height >= max_terminal_height {
                    break;
                }
            }
        }
    }

    let tick_rate = Duration::from_millis(1000 / 60);
    let mut current_time = Instant::now();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    loop {
        emulator.tick();
        terminal.draw(|f| ui(f, &emulator.screen))?;
        emulator.keypad = [false; chip8_core::KEYPAD_SIZE * chip8_core::KEYPAD_SIZE];

        if current_time.elapsed() >= tick_rate {
            if emulator.delay_timer > 0 {
                emulator.delay_timer -= 1;
            }
            if emulator.sound_timer > 0 {
                if emulator.sound_timer == 1 {
                    sink.stop();
                    sink.append(SineWave::new(440.0).take_duration(Duration::from_millis(100)));
                }
                emulator.sound_timer -= 1;
            }
            current_time = Instant::now();
        }

        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }
                if emulator.verbose {
                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .create(true)
                        .open("log.txt")?;
                    writeln!(file, "{:?} press detected", key.code)?;
                }
                match key.code {
                    KeyCode::Esc => break Ok(true),
                    _ => {
                        if let Some(&key) = key_mapping.get(&key.code) {
                            emulator.keypad[key] = true;
                        }
                    }
                }
            }
        }
    }
}
