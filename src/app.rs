use failure::{err_msg, format_err};
use std::str::FromStr;
use std::{
    cmp::{max, min},
    collections::HashSet,
    time::Instant,
};
use tui::layout::Rect;

pub struct App {
    pub input: Vec<char>,
    pub input_idx: usize,
    pub input_cursor_position: u16,
    pub tasks: Vec<String>,
    pub size: Rect,
}

impl App {
    pub fn new() -> App {
        App {
            input: vec![],
            input_idx: 0,
            input_cursor_position: 0,
            tasks: Vec::new(),
            size: Rect::default(),
        }
    }
}