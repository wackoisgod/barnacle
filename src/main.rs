mod app;
mod config;
mod event;
mod gist;
mod ui;

use crate::event::Key;
use app::{App, AppMode, VimCommandBarResult, WorkItem};
use backtrace::Backtrace;
use clap::App as ClapApp;
use config::ClientConfig;
use std::error::Error;
use std::str::SplitWhitespace;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::Print,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::{
    io::{self, stdout, Write},
    panic::{self, PanicInfo},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
extern crate serde_json;

fn close_application() -> Result<(), failure::Error> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

pub fn on_down_press_handler<T>(
    selection_data: &[T],
    selection_index: Option<usize>,
) -> usize {
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

pub fn on_up_press_handler<T>(
    selection_data: &[T],
    selection_index: Option<usize>,
) -> usize {
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

fn panic_hook(info: &PanicInfo<'_>) {
    if cfg!(debug_assertions) {
        let location = info.location().unwrap();

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        let stacktrace: String =
            format!("{:?}", Backtrace::new()).replace('\n', "\n\r");

        disable_raw_mode().unwrap();
        execute!(
            io::stdout(),
            LeaveAlternateScreen,
            Print(format!(
                "thread '<unnamed>' panicked at '{}', {}\n\r{}",
                msg, location, stacktrace
            )),
            DisableMouseCapture
        )
        .unwrap();
    }
}

pub fn parse_text_parts(parts: &mut SplitWhitespace) -> Option<String> {
    // Parse text parts and nest them together
    let mut text_raw = String::new();

    while let Some(text_part) = parts.next() {
        if !text_raw.is_empty() {
            text_raw.push_str(" ");
        }

        text_raw.push_str(text_part);
    }

    if text_raw.is_empty() {
        None
    } else {
        Some(text_raw)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    panic::set_hook(Box::new(|info| {
        panic_hook(info);
    }));

    ClapApp::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .usage("Press `?` while running the app to see keybindings")
        .after_help(
            "Your Github Gist ID and Client Secret are stored in $HOME/.config/barnacle/client.yml",
        )
        .get_matches();

    let mut client_config = ClientConfig::new();
    client_config.load_config()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = event::Events::new();

    let mut app = App::new();
    app.client_config = client_config;

    app.init().await;
    app.sync().await;

    terminal.clear()?;

    loop {
        if let Ok(size) = terminal.backend().size() {
            app.size = size;
        };

        terminal.draw(|mut f| {
            ui::draw_core_layout(&mut f, &app);
        })?;

        match app.mode {
            AppMode::Global => match terminal.hide_cursor() {
                Ok(_r) => {}
                Err(_e) => {}
            },
            _ => match terminal.show_cursor() {
                Ok(_r) => {}
                Err(_e) => {}
            },
        };

        let cursor_offset = if app.size.height > ui::SMALL_TERMINAL_HEIGHT {
            2
        } else {
            1
        };

        let hieght = terminal.get_frame().size().height;

        // Put the cursor back inside the input box
        terminal.set_cursor(
            cursor_offset + app.get_cursor_position(),
            hieght - 3,
        )?;

        io::stdout().flush().ok();

        app.tasks
            .sort_by(|a, b| a.status.partial_cmp(&b.status).unwrap());

        match events.next()? {
            event::Event::Input(key) => {
                if key == Key::Ctrl('c') {
                    close_application()?;
                    break;
                }

                if key == Key::Esc {
                    app.insert_bar.clear();
                    app.command_bar.clear();
                    app.mode = AppMode::Global;
                    terminal.hide_cursor()?;
                    app.selected_index = 0;
                }

                match app.mode {
                    AppMode::Global => {
                        match key {
                            Key::Char('s') => {
                                if let Some(w) =
                                    app.tasks.get_mut(app.selected_index)
                                {
                                    w.start()
                                }
                            }
                            Key::Char('f') => {
                                if let Some(w) =
                                    app.tasks.get_mut(app.selected_index)
                                {
                                    w.fin()
                                }
                            }
                            Key::Char('w') => {
                                if let Some(w) =
                                    app.tasks.get_mut(app.selected_index)
                                {
                                    w.wont_fix()
                                }
                            }
                            Key::Char('i') => app.mode = AppMode::Insert,
                            Key::Char(':') => {
                                app.mode = AppMode::Command;
                                app.command_bar.handle_input(key);
                            }
                            Key::Char('o') => {
                                app.save_project(false);
                            }
                            Key::Up => {
                                let next_index = on_up_press_handler(
                                    &app.tasks,
                                    Some(app.selected_index),
                                );
                                app.selected_index = next_index;
                            }
                            Key::Down => {
                                let next_index = on_down_press_handler(
                                    &app.tasks,
                                    Some(app.selected_index),
                                );
                                app.selected_index = next_index;
                            }
                            Key::Ctrl('d') => {
                                // Move these to global things
                                app.tasks.remove(app.selected_index);
                                let next_index = on_up_press_handler(
                                    &app.tasks,
                                    Some(app.selected_index),
                                );

                                app.selected_index = next_index;
                            }
                            _ => {}
                        };
                    }
                    AppMode::Insert => match app.insert_bar.handle_input(key) {
                        VimCommandBarResult::Finished(task) => {
                            let mut work_item = WorkItem::new();
                            work_item.content = Some(task);
                            app.tasks.push(work_item);
                            app.tasks.sort_by(|a, b| {
                                a.status.partial_cmp(&b.status).unwrap()
                            });
                            app.selected_index = app.tasks.len() - 1;
                            app.mode = AppMode::Global;
                            app.insert_bar.clear();
                            app.save_project(false);
                        }
                        VimCommandBarResult::Aborted => {
                            app.mode = AppMode::Global
                        }
                        _ => {}
                    },
                    AppMode::Command => {
                        match app.command_bar.handle_input(key) {
                            // I don't like this code here but for now its fine
                            VimCommandBarResult::Finished(cmd) => {
                                let mut tokens = cmd.split_whitespace();
                                let c = match tokens.next() {
                                    Some(c) => &c[1..],
                                    None => return Ok(()),
                                };

                                match c {
                                    "q" => {
                                        close_application()?;
                                        break;
                                    }
                                    "t" => {
                                        // these are task based commands?
                                        let task_action = match tokens.next() {
                                            Some(c) => c,
                                            None => return Ok(()),
                                        };

                                        match task_action {
                                            "mod" => {
                                                // this a modification actions so what we do is replace the current selected item
                                                let index =
                                                    tokens.next().unwrap();
                                                match parse_text_parts(
                                                    &mut tokens,
                                                ) {
                                                    Some(v) => app
                                                        .update_work_item_text(
                                                            index
                                                                .parse::<usize>(
                                                                )
                                                                .unwrap(),
                                                            &v,
                                                        ),
                                                    _ => {}
                                                };
                                            }
                                            _task_action @ _ => {}
                                        }
                                    }
                                    "p" => {
                                        let task_action = match tokens.next() {
                                            Some(c) => c,
                                            None => return Ok(()),
                                        };

                                        match task_action {
                                            "new" => {
                                                // this a modification actions so what we do is replace the current selected item
                                                let name =
                                                    tokens.next().unwrap();
                                                app.new_project(&String::from(
                                                    name,
                                                ))
                                                .await;
                                            }
                                            "open" => {
                                                // this a modification actions so what we do is replace the current selected item
                                                let name =
                                                    tokens.next().unwrap();
                                                app.select_project(
                                                    &String::from(name),
                                                );
                                                app.sync().await;
                                            }
                                            _task_action @ _ => {}
                                        }
                                    }
                                    _c @ _ => {}
                                };
                            }
                            VimCommandBarResult::Aborted => {
                                app.mode = AppMode::Global
                            }
                            _ => {}
                        }
                    }
                };
            }
            event::Event::Tick => {
                // we need to do somet stuff herer ?
            }
        }
    }

    Ok(())
}
