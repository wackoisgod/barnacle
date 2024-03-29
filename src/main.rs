mod app;
mod config;
mod event;
mod gist;
mod ui;

use crate::event::KeyCode;
use anyhow::Result;
use app::{App, AppMode, VimCommand, VimCommandBarResult, WorkItem};
use backtrace::Backtrace;
use clap::App as ClapApp;
use config::ClientConfig;
use event::{EventIterator, KeyModifiers};
use std::error::Error;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::Print,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen, is_raw_mode_enabled,
    },
};
use std::{
    io::{self, stdout, Write},
    panic::{self, PanicInfo},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal, layout::{Direction, Layout, Constraint},
};
extern crate serde_json;

fn close_application() -> Result<()> {
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
    if !is_raw_mode_enabled()? {
        enable_raw_mode()?;
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.show_cursor()?;

    let mut events = event::get_events();

    let mut app = App::new();
    app.client_config = client_config;

    app.init().await;
    app.sync().await;

    terminal.clear()?;

    loop {
        let parent_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1), 
                    Constraint::Length(3)
                ].as_ref())
            .margin(0);

        if let Ok(size) = terminal.backend().size() {
            app.size = size;
        };

        terminal.draw(|mut f| {
            ui::draw_core_layout(&mut f, &app, &parent_layout);
        })?;

        io::stdout().flush().ok();

        let mut current_view = app.get_view();
        current_view.sort_by(|a, b| a.status.partial_cmp(&b.status).unwrap());

        let keyEvent = events.next_event()?;
        let _keyHandles = match keyEvent.code {
            KeyCode::Char('d') if keyEvent.modifiers.contains(KeyModifiers::CONTROL) => {
                {
                    close_application()?;
                    break;
                }
            },
            KeyCode::Char('c') => if let AppMode::Global = app.mode {
                app.insert_bar.clear();
                app.command_bar.clear();
                app.mode = AppMode::Global;
                terminal.hide_cursor()?;
                app.selected_index = 0;
                continue;
            },
            KeyCode::Esc => {
                app.insert_bar.clear();
                app.command_bar.clear();
                app.mode = AppMode::Global;
                terminal.hide_cursor()?;
                app.selected_index = 0;
                continue;
            }
            _=>{}
        };

        match app.mode {
            AppMode::Global => match keyEvent.code {
                KeyCode::Char('s') => {
                    if let Some(w) =
                        current_view.get_mut(app.selected_index)
                    {
                        app.start_task(&w.id.as_ref().unwrap())
                    }
                },
                KeyCode::Char('x') => {
                    app.fix_all_work_items();
                }
                KeyCode::Char('f') => {
                    if let Some(w) =
                        current_view.get_mut(app.selected_index)
                    {
                        app.finish_task(&w.id.as_ref().unwrap())
                    }
                }
                KeyCode::Char('w') => {
                    if let Some(w) =
                        current_view.get_mut(app.selected_index)
                    {
                        app.wont_task(&w.id.as_ref().unwrap())
                    }
                }
                KeyCode::Char('d') => {
                    if let Some(w) =
                        current_view.get_mut(app.selected_index)
                    {
                        app.remove_task(&w.id.as_ref().unwrap())
                    }
                }
                KeyCode::Char('p') => {
                    if app.register.is_some() {
                        app.add_task(
                            app.register.as_ref().unwrap().clone(),
                        );
                    }
                }
                KeyCode::Char('r') => {
                    app.sync().await;
                }
                KeyCode::Char('i') => app.mode = AppMode::Insert,
                KeyCode::Char(':') => {
                    app.mode = AppMode::Command;
                    app.command_bar.handle_input(keyEvent);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let next_index = on_up_press_handler(
                        &app.tasks,
                        Some(app.selected_index),
                    );
                    app.selected_index = next_index;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let next_index = on_down_press_handler(
                        &app.tasks,
                        Some(app.selected_index),
                    );
                    app.selected_index = next_index;
                }
                _=>{}
            }
            AppMode::Insert => match app.insert_bar.handle_input(keyEvent) {
                VimCommandBarResult::Finished(task) => {
                    let mut work_item = WorkItem::new();
                    work_item.content = Some(task);
                    app.add_task(work_item);
                }
                VimCommandBarResult::Aborted => {
                    app.mode = AppMode::Global
                }
                _ => {}
            },
            AppMode::Command => {
                match app.command_bar.handle_input(keyEvent) {
                    VimCommandBarResult::Finished(cmd) => {
                        let c = VimCommand::from(cmd);
                        match c {
                            VimCommand::Quit => {
                                close_application()?;
                                break;
                            }
                            VimCommand::ProjectSaveAndQuit => {
                                app.save_project(true, false);
                                close_application()?;
                                break;
                            }
                            VimCommand::ProjectSave => {
                                app.save_project(false, false);
                            }
                            VimCommand::TaskRename(index, content) => {
                                if let Some(w) =
                                    current_view.get_mut(index)
                                {
                                    app.update_work_item_text(
                                        &w.id.as_ref().unwrap(),
                                        &content,
                                    );
                                }
                            }
                            VimCommand::TaskDelete(index) => {
                                if let Some(w) =
                                    current_view.get_mut(index)
                                {
                                    app.remove_task(
                                        &w.id.as_ref().unwrap(),
                                    )
                                }
                            }
                            VimCommand::ProjectNew(name) => {
                                app.new_project(&name).await;
                            }
                            VimCommand::ProjectOpen(name) => {
                                app.select_project(&name).await;
                                app.sync().await;
                            }
                            VimCommand::ShowFinished(value) => {
                                app.client_config.show_finished =
                                    Some(value);
                                let _ = app.client_config.save_config();
                            }
                            VimCommand::ShowToday(value) => {
                                app.client_config.show_today =
                                    Some(value);
                                let _ = app.client_config.save_config();
                            }

                            VimCommand::TaskSetPriority(_, _) => {}
                            VimCommand::None => {}
                        };
                        app.mode = AppMode::Global
                    }
                    VimCommandBarResult::Aborted => {
                        app.mode = AppMode::Global
                    }
                _ => {}
                }
            }
        };
    }

    Ok(())
}
