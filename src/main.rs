mod app;
mod event;
mod ui;

use crate::event::Key;
use app::{App};
use clap::App as ClapApp;
use std::error::Error;
use crossterm::{
    cursor::MoveTo,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::{
    cmp::{max, min},
    io::{self, stdout, Write},
    panic::{self, PanicInfo},
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::convert::TryInto;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

fn close_application() -> Result<(), failure::Error> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn compute_character_width(character: char) -> u16 {
    UnicodeWidthChar::width(character)
        .unwrap()
        .try_into()
        .unwrap()
}

fn main() -> Result<(), Box<dyn Error>> {

    ClapApp::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .usage("Press `?` while running the app to see keybindings")
        .after_help("Your Github Gist ID and Client Secret are stored in $HOME/.config/barnacle/client.yml")
        .get_matches();

        // Terminal initialization
        enable_raw_mode()?;

        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        let events = event::Events::new();

        let mut app = App::new();

        terminal.clear()?;

        loop {
            if let Ok(size) = terminal.backend().size() {
                app.size = size;

                let potential_limit = max((app.size.height as i32) - 13, 0) as u32;
                let max_limit = min(potential_limit, 50);
            };

            terminal.show_cursor()?;

            terminal.draw(|mut f| {            
                ui::draw_core_layout(&mut f, &app);
            })?;

            let cursor_offset = if app.size.height > ui::SMALL_TERMINAL_HEIGHT {
                2
            } else {
                1
            };

            let hieght = terminal.get_frame().size().height;

            // Put the cursor back inside the input box
            terminal.set_cursor(
                cursor_offset + app.input_cursor_position,
                hieght - 2 ,
            )?;

            io::stdout().flush().ok();

            match events.next()? {
                event::Event::Input(key) => {
                    if key == Key::Ctrl('c') {
                        close_application()?;
                        break;
                    }
                    match key {
                        Key::Ctrl('u') => {
                            app.input = vec![];
                            app.input_idx = 0;
                            app.input_cursor_position = 0;
                        }
                        Key::Ctrl('a') => {
                            app.input_idx = 0;
                            app.input_cursor_position = 0;
                        }
                        Key::Enter => {
                            app.tasks.push(app.input.iter().collect());
                        }
                        Key::Char(c) => {
                            app.input.insert(app.input_idx, c);
                            app.input_idx += 1;
                            app.input_cursor_position += compute_character_width(c);
                        }
                        Key::Backspace => {
                            if !app.input.is_empty() && app.input_idx > 0 {
                                let last_c = app.input.remove(app.input_idx - 1);
                                app.input_idx -= 1;
                                app.input_cursor_position -= compute_character_width(last_c);
                            }
                        }
                        Key::Delete => {
                            if !app.input.is_empty() && app.input_idx < app.input.len() {
                                app.input.remove(app.input_idx);
                            }
                        }
                        _ => {}
                    }                   
                }
                event::Event::Tick => {
                   
                }
            }
        }

    Ok(())
}
