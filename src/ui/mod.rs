use super::app::{App, AppMode, ItemStatus, WorkItem};



use ratatui::backend::{Backend};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};

use ratatui::Frame;

use chrono::offset::Local;
use std::fmt::Write;

pub const SMALL_TERMINAL_HEIGHT: u16 = 45;

#[derive(PartialEq)]
pub enum ColumnId {
    None,
    Id,
    Content,
    Days,
}

impl Default for ColumnId {
    fn default() -> Self {
        ColumnId::None
    }
}

pub struct TableHeader<'a> {
    items: Vec<TableHeaderItem<'a>>,
}

impl TableHeader<'_> {
    pub fn get_index(&self, id: ColumnId) -> Option<usize> {
        self.items.iter().position(|item| item.id == id)
    }
}

#[derive(Default)]
pub struct TableHeaderItem<'a> {
    id: ColumnId,
    text: &'a str,
    width: u16,
}

#[allow(dead_code)]
pub struct TableItem<'a> {
    id: String,
    org_item: &'a WorkItem,
    format: Vec<String>,
}

pub fn get_percentage_width(width: u16, percentage: f32) -> u16 {
    let padding = 3;
    let width = width - padding;
    (f32::from(width) * percentage) as u16
}

pub fn draw_input_and_help_box<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
) where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [Constraint::Percentage(95), Constraint::Percentage(5)].as_ref(),
        )
        .split(layout_chunk);

    let mut input_string = String::new();
    let (_a, b) = match app.mode {
        AppMode::Global => (write!(input_string, "").unwrap(), 0),
        AppMode::Command => (
            write!(input_string, "{}", app.command_bar.buffer()).unwrap(),
            app.command_bar.input_cursor_position(),
        ),
        AppMode::Insert => (
            write!(input_string, "{}", app.insert_bar.buffer()).unwrap(),
            app.insert_bar.input_cursor_position(),
        ),
    };

    let title = format!("{} Mode:", app.mode);

    
    let p1 = Paragraph::new(Span::raw(input_string))
        .block(Block::default().borders(Borders::ALL).title(Span::raw(title)))
        .style(match app.mode {
            AppMode::Global => Style::default(),
            AppMode::Command => Style::default().fg(Color::Yellow),
            AppMode::Insert => Style::default().fg(Color::Green),
        });

    f.render_widget(p1, chunks[0]);

    let p2 = Paragraph::new(Span::raw(b.to_string()))
        .block(Block::default().borders(Borders::ALL).title("Help:"));

    f.render_widget(p2, chunks[1]);
}

pub fn draw_task_list<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(layout_chunk);

    let header = TableHeader {
        items: vec![
            TableHeaderItem {
                id: ColumnId::Id,
                text: "Id",
                width: get_percentage_width(layout_chunk.width, 0.2 / 9.0),
            },
            TableHeaderItem {
                id: ColumnId::Content,
                text: "Content",
                width: get_percentage_width(layout_chunk.width, 8.0 / 9.0),
            },
            TableHeaderItem {
                text: "Started",
                width: get_percentage_width(layout_chunk.width, 0.6 / 9.0),
                ..Default::default()
            },
            TableHeaderItem {
                id: ColumnId::Days,
                text: "Days",
                width: get_percentage_width(layout_chunk.width, 1.6 / 6.0),
            },
        ],
    };

    let mut current_view = app.get_view();

    current_view.sort_by(|a, b| a.status.partial_cmp(&b.status).unwrap());

    let messages = current_view
        .iter()
        .enumerate()
        .map(|(i, m)| TableItem {
            id: i.to_string(),
            org_item: m,
            format: vec![
                i.to_string(),
                m.content.as_ref().unwrap().to_string(),
                if let Some(start_time) = m.started_time {
                    start_time.format("%Y-%m-%d").to_string()
                } else {
                    "-".to_string()
                },
                if m.finished_time.is_some() {
                    "-".to_string()
                } else {
                    Local::now()
                        .signed_duration_since(m.created_time)
                        .num_days()
                        .to_string()
                },
            ],
        })
        .collect::<Vec<TableItem>>();
    draw_table(
        f,
        app,
        chunks[0],
        &header,
        &messages,
        app.selected_index,
        (false, false),
    );
}

pub fn draw_core_layout<B>(f: &mut Frame<B>, app: &App, layout: &Layout)
where
    B: Backend,
{
    let parent_layout: std::rc::Rc<[Rect]> = layout.split(f.size());

    draw_task_list(f, app, parent_layout[0]);
    draw_input_and_help_box(f, app, parent_layout[1]);

    match app.mode {
        AppMode::Global => {
            {}
        },
        _ => {
            // Put the cursor back inside the input box
            f.set_cursor(
                1 + app.get_cursor_position(),
                parent_layout[1].y + 1,
            );
        },
    };

}

fn draw_table<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    table_layout: &TableHeader,
    items: &[TableItem], // The nested vector must have the same length as the `header_columns`
    selected_index: usize,
    _highlight_state: (bool, bool),
) where
    B: Backend,
{
    let header = table_layout;

    let selected_style = Style::default()
        .fg(Color::LightBlue)
        .add_modifier(Modifier::BOLD);

    let padding = 5;
    let offset = layout_chunk
        .height
        .checked_sub(padding)
        .and_then(|height| selected_index.checked_sub(height as usize))
        .unwrap_or(0);

    let rows = items.iter().skip(offset).enumerate().map(|(i, item)| {
        let formatted_row = item.format.clone();
        let mut style = Style::default(); // default styling

        // TODO: May want to change the style if its been sitting to many days
        if let Some(_title_idx) = header.get_index(ColumnId::Content) {
            match item.org_item.status {
                ItemStatus::WontFix => {
                    style = style.fg(Color::Red).add_modifier(Modifier::CROSSED_OUT)
                }
                ItemStatus::Started => style = style.fg(Color::LightGreen),
                ItemStatus::Finished => {
                    style = style.fg(Color::Rgb(149, 66, 245))
                }
                _ => {}
            }
        }

        // TODO: May want to change the style if its been sitting to many days
        if header.get_index(ColumnId::Days).is_some() {}

        // Next check if the item is under selection.
        if Some(i) == selected_index.checked_sub(offset) {
            style = selected_style;
        }

        Row::new(formatted_row.into_iter()).style(style)
    });

    let widths = header
        .items
        .iter()
        .map(|h| Constraint::Length(h.width))
        .collect::<Vec<ratatui::layout::Constraint>>();

    let title = format!(
        "{}({}):",
        app.current_project.as_ref().unwrap_or(&"Tasks".to_string()),
        app.filter,
    );

    let h = Row::new(header.items.iter().map(|h| h.text.clone()));

    let t1 = Table::new(rows)
        .header(h)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title(Span::raw(title)),
        )
        .style(Style::default())
        .widths(&widths);

    f.render_widget(t1, layout_chunk);
}
