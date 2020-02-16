extern crate chrono;
use self::AppFilterMode::*;
use super::config::ClientConfig;
use super::{gist::get_gist_file, gist::ListGist};
use chrono::prelude::*;
use num_enum::TryFromPrimitive;
use serde_derive::{Deserialize, Serialize};
use std::{
    fmt,
};
use tui::layout::Rect;
use ropey::Rope;
use crate::event::Key;
use std::convert::TryInto;
use unicode_width::{UnicodeWidthChar};
use std::fmt::Write;


fn compute_character_width(character: char) -> u16 {
    UnicodeWidthChar::width(character)
        .unwrap()
        .try_into()
        .unwrap()
}

pub struct VimCommandBar {
    buffer: Rope,
    input_idx: usize,
    input_cursor_position: u16,
}

pub enum VimCommandBarResult {
    StillEditing,
    Aborted,
    Finished(String),
}

impl VimCommandBar {
    pub fn new() -> Self {
        Self {
            buffer: Rope::new(),
            input_idx: 0,
            input_cursor_position: 0,
        }
    }

    pub fn buffer(&self) -> &Rope {
        &self.buffer
    }

    pub fn input_idx(&self) -> usize {
        self.input_idx
    }

    pub fn input_cursor_position(&self) -> u16 {
        self.input_cursor_position
    }

    pub fn handle_input(&mut self, key: Key) -> VimCommandBarResult {
        match key {
            Key::Char(c) => {
                self.buffer.insert_char(self.input_idx, c);
                self.input_idx += 1;
                self.input_cursor_position += compute_character_width(c);
                VimCommandBarResult::StillEditing
            }
            Key::Enter => {
                let mut result = String::new();
                write!(result, "{}", self.buffer).unwrap();
                self.clear();
                VimCommandBarResult::Finished(result)
            },
            Key::Backspace => {
                match (self.buffer.len_chars() == 0, self.input_idx == 0) {
                    (true, _) => return VimCommandBarResult::Aborted,
                    (false, true) => {},
                    (false, false) => {
                        self.input_cursor_position -= compute_character_width(self.buffer.char(self.input_idx - 1));
                        self.buffer.remove(self.input_idx - 1..self.input_idx);
                        self.input_idx -= 1;
                    },
                }                
                VimCommandBarResult::StillEditing
            },
            Key::Delete => {
                let len = self.buffer.len_chars();
                if self.input_idx < len {
                    self.buffer.remove(self.input_idx..self.input_idx + 1);
                }
                VimCommandBarResult::StillEditing
            },
            Key::Ctrl('u') => {
                self.clear();
                VimCommandBarResult::StillEditing               
            },
            Key::Ctrl('a') => {
                self.goto_being();
                VimCommandBarResult::StillEditing               
            },
            _ => VimCommandBarResult::StillEditing,
        }
    }

    pub fn clear(&mut self) {
        self.buffer = Rope::new();
        self.input_idx = 0;
        self.input_cursor_position = 0;
    }

    pub fn goto_being(&mut self) {
        self.input_idx = 0;
        self.input_cursor_position = 0;
    }
}

pub struct VimInsertBar {
    buffer: Rope,
    input_idx: usize,
    input_cursor_position: u16,
}

impl VimInsertBar {
    pub fn new() -> Self {
        Self {
            buffer: Rope::new(),
            input_idx: 0,
            input_cursor_position: 0,
        }
    }

    pub fn buffer(&self) -> &Rope {
        &self.buffer
    }

    pub fn input_idx(&self) -> usize {
        self.input_idx
    }

    pub fn input_cursor_position(&self) -> u16 {
        self.input_cursor_position
    }

    pub fn handle_input(&mut self, key: Key) -> VimCommandBarResult {
        match key {
            Key::Left => {
                if self.buffer.len_chars() > 0 && self.input_idx > 0 {
                    let last_c = self.buffer.char(self.input_idx - 1);                    
                    self.input_idx -= 1;
                    self.input_cursor_position -= compute_character_width(last_c);
                }
                VimCommandBarResult::StillEditing
            },
            Key::Char(c) => {
                self.buffer.insert_char(self.input_idx, c);
                self.input_idx += 1;
                self.input_cursor_position += compute_character_width(c); 
                VimCommandBarResult::StillEditing
            },
            Key::Enter => {
                let mut result = String::new();
                write!(result, "{}", self.buffer).unwrap();
                self.clear();
                VimCommandBarResult::Finished(result)
            },
            Key::Backspace => {
                match (self.buffer.len_chars() == 0, self.input_idx == 0) {
                    (true, _) => return VimCommandBarResult::Aborted,
                    (false, true) => {},
                    (false, false) => {
                        self.input_cursor_position -= compute_character_width(self.buffer.char(self.input_idx - 1));
                        self.buffer.remove(self.input_idx - 1..self.input_idx);
                        self.input_idx -= 1;
                    },
                }
                VimCommandBarResult::StillEditing
            },
            Key::Delete => {
                let len = self.buffer.len_chars();
                if self.input_idx < len {
                    self.buffer.remove(self.input_idx..self.input_idx + 1);
                }
                VimCommandBarResult::StillEditing
            },
            Key::Ctrl('u') => {
                self.clear();                
                VimCommandBarResult::StillEditing
            },
            Key::Ctrl('a') => {
                self.goto_being();
                VimCommandBarResult::StillEditing
            },
            _ => VimCommandBarResult::StillEditing,
        }
    }

    pub fn clear(&mut self) {
        self.buffer = Rope::new();
        self.input_idx = 0;
        self.input_cursor_position = 0;
    }

    pub fn goto_being(&mut self) {
        self.input_idx = 0;
        self.input_cursor_position = 0;
    }
}


#[derive(Clone, Eq, Ord, PartialEq, PartialOrd, Debug, Copy, Serialize, Deserialize)]
pub enum ItemStatus {
    Started,
    Finished,
    UnStarted,
    WontFix,
}

#[derive(Clone, PartialEq, Debug, Copy, TryFromPrimitive)]
#[repr(usize)]
pub enum AppFilterMode {
    All,
    Started,
    Finished,
    WontFix,
}

impl AppFilterMode {
    pub fn iterator() -> impl Iterator<Item = AppFilterMode> {
        [All, Started, Finished, WontFix].iter().copied()
    }
}

impl fmt::Display for AppFilterMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AppFilterMode::All => write!(f, "All"),
            AppFilterMode::Started => write!(f, "Started"),
            AppFilterMode::Finished => write!(f, "Finished"),
            AppFilterMode::WontFix => write!(f, "WontFix"),
        }
    }
}

pub enum AppMode {
    Insert,
    Command,
    Global,
}

impl fmt::Display for AppMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AppMode::Insert => write!(f, "Insert"),
            AppMode::Command => write!(f, "Command"),
            AppMode::Global => write!(f, "Global"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: Option<String>,
    pub content: Option<String>,
    pub status: ItemStatus,
    #[serde(with = "normal_date_format")]
    pub created_time: DateTime<Local>,
    #[serde(with = "option_date_format")]
    pub started_time: Option<DateTime<Local>>,
    #[serde(with = "option_date_format")]
    pub finished_time: Option<DateTime<Local>>,
}

mod normal_date_format {
    use chrono::{DateTime, Local, TimeZone};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Local
            .datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

mod option_date_format {
    use chrono::{DateTime, Local, TimeZone};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(date: &Option<DateTime<Local>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(d) = date {
            serializer.serialize_str(&d.format(FORMAT).to_string())
        } else {
            serializer.serialize_unit()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Local>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer) {
            Ok(_r) => Ok(Some(Local.datetime_from_str(&_r, FORMAT).unwrap())),
            Err(_e) => Ok(None),
        }
    }
}

impl WorkItem {
    pub fn new() -> WorkItem {
        WorkItem {
            id: None,
            content: None,
            status: ItemStatus::UnStarted,
            created_time: Local::now(),
            started_time: None,
            finished_time: None,
        }
    }

    pub fn start(&mut self) {
        self.started_time = Some(Local::now());
        self.status = ItemStatus::Started;
        self.finished_time = None;
    }

    pub fn fin(&mut self) {
        self.finished_time = Some(Local::now());
        self.status = ItemStatus::Finished;
    }

    pub fn wont_fix(&mut self) {
        self.status = ItemStatus::WontFix;
    }

    pub fn is_valid_for_mode(&self, a: AppFilterMode) -> bool {
        match a {
            AppFilterMode::All => true,
            AppFilterMode::Started => self.status == ItemStatus::Started,
            AppFilterMode::Finished => self.status == ItemStatus::Finished,
            AppFilterMode::WontFix => self.status == ItemStatus::WontFix,
        }
    }
}

pub struct App {
    pub tasks: Vec<WorkItem>,
    pub size: Rect,
    pub selected_index: usize,
    pub filter: AppFilterMode,
    pub write_mode: bool,
    pub client_config: ClientConfig,
    pub command_bar: VimCommandBar,
    pub insert_bar: VimInsertBar,
    pub mode: AppMode,
}

impl App {
    pub fn new() -> App {
        App {
            tasks: Vec::new(),
            size: Rect::default(),
            selected_index: 0,
            filter: AppFilterMode::All,
            write_mode: true,
            client_config: Default::default(),
            command_bar: VimCommandBar::new(),
            insert_bar: VimInsertBar::new(),
            mode: AppMode::Global
        }
    }

    pub fn sync(&mut self) {
        let list_gist = ListGist::get_update_list_gist(&self.client_config.client_secret).unwrap();
        let file = list_gist
            .get_url_gist_file(&self.client_config.client_id)
            .unwrap();
        let data = get_gist_file(&file, &self.client_config.client_secret).unwrap();
        let actual_data: Vec<WorkItem> = serde_json::from_str(&data).unwrap();
        self.tasks = actual_data;
    }

    pub fn get_cursor_position(&self) -> u16 {
        match self.mode {
            AppMode::Global => 0,
            AppMode::Command => self.command_bar.input_cursor_position(),
            AppMode::Insert => self.insert_bar.input_cursor_position()
        }
    }
}
