use super::{
    app::{
        App,
    }
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph, Row, SelectableList, Table, Text, Widget, List},
    Frame,
};

pub const SMALL_TERMINAL_HEIGHT: u16 = 45;

pub fn draw_input_and_help_box<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(95), Constraint::Percentage(5)].as_ref())
        .split(layout_chunk);

    let input_string: String = app.input.iter().collect();
    Paragraph::new([Text::raw(&input_string)].iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("New Task:"),
        )
        .render(f, chunks[0]);

    Paragraph::new([Text::raw(app.input_cursor_position.to_string())].iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help:"),
        )
        .render(f, chunks[1]);
}

pub fn draw_task_list<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(layout_chunk);

    let messages = app
        .tasks
        .iter()
        .enumerate()
        .map(|(i, m)| Text::raw(format!("{}: {}", i, m)));

    List::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Messages"))
        .render(f, chunks[0]);
}

pub fn draw_core_layout<B>(f: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let margin = if app.size.height > SMALL_TERMINAL_HEIGHT {
        1
    } else {
        0
    };


    let parent_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
        [
            Constraint::Min(20),
            Constraint::Length(3),
        ]
        .as_ref(),
    )
    .margin(margin)
    .split(f.size());

    draw_task_list(f, app, parent_layout[0]);
    draw_input_and_help_box(f, app, parent_layout[1]);
}