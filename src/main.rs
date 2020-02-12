mod app;
mod event;
mod ui;

use crate::event::Key;
use app::{App, WorkItem, AppMode};
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
use chrono::offset::{Local};
extern crate serde_json;
use std::fs::File;
use std::io::Read;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

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

pub fn on_down_press_handler<T>(selection_data: &[T], selection_index: Option<usize>) -> usize {
    match selection_index {
        Some(selection_index) => {
            if !selection_data.is_empty() {
                let next_index = selection_index + 1;
                if next_index > selection_data.len() - 1 {
                    return 0;
                } else {
                    return next_index;
                }
            }
            0
        }
        None => 0,
    }
}

pub fn on_up_press_handler<T>(selection_data: &[T], selection_index: Option<usize>) -> usize {
    match selection_index {
        Some(selection_index) => {
            if !selection_data.is_empty() {
                if selection_index > 0 {
                    return selection_index - 1;
                } else {
                    return selection_data.len() - 1;
                }
            }
            0
        }
        None => 0,
    }
}

pub fn handler(key: Key, app: &mut App) -> Result<(), Box<dyn Error>> {
    match key {
        Key::Char('s') => {
            if let Some(w) = app.tasks.get_mut(app.selected_index) {
                w.start()
            }
        }
        Key::Char('f') => {
            if let Some(w) = app.tasks.get_mut(app.selected_index) {
                w.fin()
            }
        }
        Key::Char('w') => {
            if let Some(w) = app.tasks.get_mut(app.selected_index) {
                w.wont_fix()
            }
        }
        Key::Char('o') => {
            ::serde_json::to_writer(&File::create("data.json")?, &app.tasks)?
        }
        Key::Char('n') => {
            let mut file = File::open("data.json")?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
        
            // Deserialize and print Rust data structure.
            let data: Vec<WorkItem> = serde_json::from_str(&contents)?;
            app.tasks = data;
        }
        _=> {}
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {

    ClapApp::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .usage("Press `?` while running the app to see keybindings")
        .after_help("Your Github Gist ID and Client Secret are stored in $HOME/.config/barnacle/client.yml")
        .get_matches();

        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        enable_raw_mode()?;

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

            terminal.draw(|mut f| {            
                ui::draw_core_layout(&mut f, &app);
            })?;

            if app.write_mode {
                match terminal.show_cursor() {
                    Ok(_r) => {}
                    Err(_e) => {}
                };
            } else {
                match terminal.hide_cursor() {
                    Ok(_r) => {}
                    Err(_e) => {}
                };
            }

            let cursor_offset = if app.size.height > ui::SMALL_TERMINAL_HEIGHT {
                2
            } else {
                1
            };

            let hieght = terminal.get_frame().size().height;

            // Put the cursor back inside the input box
            terminal.set_cursor(
                cursor_offset + app.input_cursor_position,
                hieght - 3,
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
                            app.input.drain(..);
                        }
                        Key::Ctrl('a') => {
                            app.input_idx = 0;
                            app.input_cursor_position = 0;
                        }
                        Key::Ctrl('d') => {
                            app.tasks.remove(app.selected_index);
                            let next_index = on_up_press_handler(
                                &app.tasks,
                                Some(app.selected_index),
                            );

                            app.selected_index = next_index;
                        }
                        Key::Left => {
                            if app.write_mode {
                                if !app.input.is_empty() && app.input_idx > 0 {
                                    let last_c = app.input[app.input_idx - 1];
                                    app.input_idx -= 1;
                                    app.input_cursor_position -= compute_character_width(last_c);
                                }
                            }
                            else 
                            {
                                let next_index = on_up_press_handler(
                                    &AppMode::iterator().collect::<Vec<AppMode>>(),
                                    Some(app.mode as usize),
                                );
                                
                                match AppMode::try_from(next_index) {
                                    Ok(size) => {
                                        app.mode = size;
                                        app.selected_index = 0;
                                    },
                                    Err(_) => app.mode = AppMode::All
                                }
                            }
                        }
                        Key::Right => {
                            if app.write_mode {
                                if app.input_idx < app.input.len() {
                                    let next_c = app.input[app.input_idx];
                                    app.input_idx += 1;
                                    app.input_cursor_position += compute_character_width(next_c);
                                }
                            }
                            else 
                            {
                                let next_index = on_down_press_handler(
                                    &AppMode::iterator().collect::<Vec<AppMode>>(),
                                    Some(app.mode as usize),
                                );

                                match AppMode::try_from(next_index) {
                                    Ok(size) => {
                                        app.mode = size;
                                        app.selected_index = 0;
                                    },
                                    Err(_) => app.mode = AppMode::All
                                }
                            }
                        }
                        Key::Enter => {
                            app.input_idx = 0;
                            app.input_cursor_position = 0;

                            let mut workItem = WorkItem::new();
                            workItem.content = Some(app.input.drain(..).collect());

                            app.tasks.push(workItem);

                            app.selected_index = app.tasks.len() - 1;
                        }
                        Key::Char(c) => {
                            if !app.write_mode && key == Key::Char('/') {
                                app.write_mode = true
                            }
                            else {
                                if app.write_mode {
                                    app.input.insert(app.input_idx, c);
                                    app.input_idx += 1;
                                    app.input_cursor_position += compute_character_width(c);    
                                }
                                else 
                                {
                                    handler(key,&mut app)?;
                                }
                            }
                        }
                        Key::Backspace => {
                            if app.write_mode {
                                if !app.input.is_empty() && app.input_idx > 0 {
                                    let last_c = app.input.remove(app.input_idx - 1);
                                    app.input_idx -= 1;
                                    app.input_cursor_position -= compute_character_width(last_c);
                                }
                            }
                        }
                        Key::Delete => {
                            if app.write_mode {
                                if !app.input.is_empty() && app.input_idx < app.input.len() {
                                    app.input.remove(app.input_idx);
                                }
                            }
                        }
                        Key::Up => {
                            if !app.write_mode {
                                let next_index = on_up_press_handler(
                                    &app.tasks,
                                    Some(app.selected_index),
                                );

                                app.selected_index = next_index;
                            }
                        }
                        Key::Esc => {
                            app.input_idx = 0;
                            app.input_cursor_position = 0;
                            app.input.drain(..);
                            app.write_mode = false;
                            terminal.hide_cursor()?;
                            app.selected_index = 0;
                        }
                        Key::Down => {
                            if !app.write_mode {
                                let next_index = on_down_press_handler(
                                    &app.tasks,
                                    Some(app.selected_index),
                                );

                                app.selected_index = next_index;
                            }
                        }
                        _ => {}
                    }                   
                }
                event::Event::Tick => {
                   // we need to do somet stuff herer ? 
                }
            }
        }

    Ok(())
}
