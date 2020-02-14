extern crate chrono;
use self::AppMode::*;
use super::config::ClientConfig;
use super::{gist::get_gist_file, gist::GistPost, gist::GistUpdate, gist::ListGist};
use chrono::prelude::*;
use num_enum::TryFromPrimitive;
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::slice::Iter;
use std::str::FromStr;
use std::{
    cmp::{max, min},
    collections::HashSet,
    fmt,
    time::Instant,
};
use tui::layout::Rect;

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd, Debug, Copy, Serialize, Deserialize)]
pub enum ItemStatus {
    Started,
    Finished,
    UnStarted,
    WontFix,
}

#[derive(Clone, PartialEq, Debug, Copy, TryFromPrimitive)]
#[repr(usize)]
pub enum AppMode {
    All,
    Started,
    Finished,
    WontFix,
}

impl AppMode {
    pub fn iterator() -> impl Iterator<Item = AppMode> {
        [All, Started, Finished, WontFix].iter().copied()
    }
}

impl fmt::Display for AppMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AppMode::All => write!(f, "All"),
            AppMode::Started => write!(f, "Started"),
            AppMode::Finished => write!(f, "Finished"),
            AppMode::WontFix => write!(f, "WontFix"),
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

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

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

    pub fn is_valid_for_mode(&self, a: AppMode) -> bool {
        match a {
            AppMode::All => true,
            AppMode::Started => self.status == ItemStatus::Started,
            AppMode::Finished => self.status == ItemStatus::Finished,
            AppMode::WontFix => self.status == ItemStatus::WontFix,
        }
    }
}

pub struct App {
    pub input: Vec<char>,
    pub input_idx: usize,
    pub input_cursor_position: u16,
    pub tasks: Vec<WorkItem>,
    pub size: Rect,
    pub selected_index: usize,
    pub mode: AppMode,
    pub write_mode: bool,
    pub client_config: ClientConfig,
}

impl App {
    pub fn new() -> App {
        App {
            input: vec![],
            input_idx: 0,
            input_cursor_position: 0,
            tasks: Vec::new(),
            size: Rect::default(),
            selected_index: 0,
            mode: AppMode::All,
            write_mode: true,
            client_config: Default::default(),
        }
    }

    pub fn sync(&mut self) {
        let list_gist = ListGist::get_update_list_gist(&self.client_config.client_secret).unwrap();
        let file = list_gist
            .get_url_gist_file(&self.client_config.client_id)
            .unwrap();
        let data = get_gist_file(&file, &self.client_config.client_secret).unwrap();
        let actualData: Vec<WorkItem> = serde_json::from_str(&data).unwrap();
        self.tasks = actualData;
    }
}
