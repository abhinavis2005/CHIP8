use chip8_core::Emulator;

use std::{error::Error, io};

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self,EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
    },
    widgets::Paragraph,
    layout::Rect,
    style::{Style, Color},
    Terminal,
    Frame
};
use std::process;
use std::fs;
fn main()-> Result<(), Box<dyn Error>>{
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
   
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;


    let _ = run_app(& mut terminal, &chip8.screen);

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
                    Paragraph::new("x")
                        .style(Style::default().fg(Color::White)),  // Adjust color if needed
                    rect,
                );
            }
        }
    }
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, screen: &[[bool; chip8_core::SCREEN_WIDTH];chip8_core::SCREEN_HEIGHT]) -> io::Result<bool>{
    loop{
        terminal.draw(|f| ui(f, screen))?;
        
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release{
                continue;
            }
            match key.code {
                KeyCode::Char('q') => break Ok(true),
                _ => {}
            }
        }
    }
}

