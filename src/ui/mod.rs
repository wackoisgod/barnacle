use super::app::{App, ItemStatus, WorkItem};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, Paragraph, Row, SelectableList, Table, Text, Widget},
    Frame,
};

use chrono::offset::Local;

pub const SMALL_TERMINAL_HEIGHT: u16 = 45;

#[derive(PartialEq)]
pub enum ColumnId {
    None,
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
        .block(Block::default().borders(Borders::ALL).title("New Task:"))
        .render(f, chunks[0]);

    Paragraph::new([Text::raw(app.input_cursor_position.to_string())].iter())
        .block(Block::default().borders(Borders::ALL).title("Help:"))
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

    let header = TableHeader {
        items: vec![
            TableHeaderItem {
                id: ColumnId::Content,
                text: "Content",
                width: get_percentage_width(layout_chunk.width, 8.0 / 9.0),
                ..Default::default()
            },
            TableHeaderItem {
                text: "Started",
                width: get_percentage_width(layout_chunk.width, 0.7 / 9.0),
                ..Default::default()
            },
            TableHeaderItem {
                id: ColumnId::Days,
                text: "Days",
                width: get_percentage_width(layout_chunk.width, 1.6 / 6.0),
                ..Default::default()
            },
        ],
    };

    let mut messages = app
        .tasks
        .iter()
        .enumerate()
        .filter(|(_u, l)| l.is_valid_for_mode(app.mode))
        .map(|(i, m)| TableItem {
            id: i.to_string(),
            org_item: m,
            format: vec![
                m.content.as_ref().unwrap().to_string(),
                if let Some(start_time) = m.started_time {
                    start_time.format("%Y-%m-%d").to_string()
                } else {
                    "-".to_string()
                },
                if let Some(_) = m.finished_time {
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
    messages.sort_by(|a,b| a.org_item.status.partial_cmp(&b.org_item.status).unwrap());
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
        .constraints([Constraint::Min(20), Constraint::Length(3)].as_ref())
        .margin(margin)
        .split(f.size());

    draw_task_list(f, app, parent_layout[0]);
    draw_input_and_help_box(f, app, parent_layout[1]);
}

fn draw_table<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    table_layout: &TableHeader,
    items: &[TableItem], // The nested vector must have the same length as the `header_columns`
    selected_index: usize,
    highlight_state: (bool, bool),
) where
    B: Backend,
{
    let header = table_layout;

    let selected_style = Style::default()
        .fg(Color::LightBlue)
        .modifier(Modifier::BOLD);

    let padding = 5;
    let offset = layout_chunk
        .height
        .checked_sub(padding)
        .and_then(|height| selected_index.checked_sub(height as usize))
        .unwrap_or(0);

    let rows = items.iter().skip(offset).enumerate().map(|(i, item)| {
        let mut formatted_row = item.format.clone();
        let mut style = Style::default(); // default styling

        // TODO: May want to change the style if its been sitting to many days
        if let Some(title_idx) = header.get_index(ColumnId::Content) {
            match item.org_item.status {
                ItemStatus::WontFix => style = style.fg(Color::Red).modifier(Modifier::CROSSED_OUT),
                ItemStatus::Started => style = style.fg(Color::LightGreen),
                ItemStatus::Finished => style = style.fg(Color::Rgb(149, 66, 245)),
                _ => {}
            }
        }

        // TODO: May want to change the style if its been sitting to many days
        if let Some(_) = header.get_index(ColumnId::Days) {}

        // Next check if the item is under selection.
        if Some(i) == selected_index.checked_sub(offset) {
            style = selected_style;
        }

        Row::StyledData(formatted_row.into_iter(), style)
    });

    let widths = header
        .items
        .iter()
        .map(|h| Constraint::Length(h.width))
        .collect::<Vec<tui::layout::Constraint>>();

    let title = format!("Tasks({}):", app.mode);

    Table::new(header.items.iter().map(|h| h.text), rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title(&title),
        )
        .style(Style::default())
        .widths(&widths)
        .render(f, layout_chunk);
}
