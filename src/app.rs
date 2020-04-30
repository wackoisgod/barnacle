extern crate chrono;
use super::config::ClientConfig;
use super::{gist::get_gist_file, gist::GistUpdate, gist::ListGist};
use crate::event::Key;
use chrono::prelude::*;
use core::str::SplitWhitespace;
use futures::executor;
use num_enum::TryFromPrimitive;
use ropey::Rope;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fmt;
use std::fmt::Write;
use tokio::task;
use tui::layout::Rect;
use unicode_width::UnicodeWidthChar;
use uuid::Uuid;

pub fn parse_text_parts(parts: &mut SplitWhitespace) -> Option<String> {
    // Parse text parts and nest them together
    let mut text_raw = String::new();

    for text_part in parts {
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

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum VimCommand {
    TaskModify(usize, String),
    TaskDelete(usize),
    TaskSetPriority(usize, usize),
    ProjectNew(String),
    ProjectOpen(String),
    ProjectSave,
    ProjectSaveAndQuit,
    Quit,
    None,
}

impl VimCommand {
    pub fn from_command(cmd: String) -> VimCommand {
        let mut tokens = cmd.split_whitespace();
        let c = match tokens.next() {
            Some(c) => &c[1..],
            None => "",
        };

        match c {
            "q" => VimCommand::Quit,
            "w" => VimCommand::ProjectSave,
            "wq" => VimCommand::ProjectSaveAndQuit,
            "tmod" => {
                let index = tokens.next().unwrap();
                let content = parse_text_parts(&mut tokens)
                    .unwrap_or(String::from("Invalid"));
                VimCommand::TaskModify(index.parse::<usize>().unwrap(), content)
            }
            "tdel" => {
                let index = tokens.next().unwrap();
                VimCommand::TaskDelete(index.parse::<usize>().unwrap())
            }
            "tp" => {
                let index = tokens.next().unwrap();
                let value = tokens.next().unwrap();
                VimCommand::TaskSetPriority(
                    index.parse::<usize>().unwrap(),
                    value.parse::<usize>().unwrap(),
                )
            }
            "popen" => {
                let name = tokens.next().unwrap();
                VimCommand::ProjectOpen(String::from(name))
            }
            "pnew" => {
                let name = tokens.next().unwrap();
                VimCommand::ProjectNew(String::from(name))
            }
            _ => VimCommand::None,
        }
    }
}

impl From<String> for VimCommand {
    fn from(command: String) -> Self {
        VimCommand::from_command(command)
    }
}

fn compute_character_width(character: char) -> u16 {
    UnicodeWidthChar::width(character)
        .unwrap()
        .try_into()
        .unwrap()
}

pub struct VimBar {
    buffer: Rope,
    input_idx: usize,
    input_cursor_position: u16,
}

pub enum VimCommandBarResult {
    StillEditing,
    Aborted,
    Finished(String),
}

impl VimBar {
    pub fn new() -> Self {
        VimBar {
            buffer: Rope::new(),
            input_idx: 0,
            input_cursor_position: 0,
        }
    }

    pub fn buffer(&self) -> &Rope {
        &self.buffer
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
                    self.input_cursor_position -=
                        compute_character_width(last_c);
                }
                VimCommandBarResult::StillEditing
            }
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
            }
            Key::Backspace => {
                match (self.buffer.len_chars() == 0, self.input_idx == 0) {
                    (true, _) => return VimCommandBarResult::Aborted,
                    (false, true) => {}
                    (false, false) => {
                        self.input_cursor_position -= compute_character_width(
                            self.buffer.char(self.input_idx - 1),
                        );
                        self.buffer.remove(self.input_idx - 1..self.input_idx);
                        self.input_idx -= 1;
                    }
                }
                VimCommandBarResult::StillEditing
            }
            Key::Delete => {
                let len = self.buffer.len_chars();
                if self.input_idx < len {
                    self.buffer.remove(self.input_idx..=self.input_idx);
                }
                VimCommandBarResult::StillEditing
            }
            Key::Ctrl('u') => {
                self.clear();
                VimCommandBarResult::StillEditing
            }
            Key::Ctrl('a') => {
                self.goto_being();
                VimCommandBarResult::StillEditing
            }
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

#[derive(
    Clone, Eq, Ord, PartialEq, PartialOrd, Debug, Copy, Serialize, Deserialize,
)]
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

    pub fn serialize<S>(
        date: &DateTime<Local>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Local>, D::Error>
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

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(
        date: &Option<DateTime<Local>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(d) = date {
            serializer.serialize_str(&d.format(FORMAT).to_string())
        } else {
            serializer.serialize_unit()
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<DateTime<Local>>, D::Error>
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

    pub fn finish(&mut self) {
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
    pub client_config: ClientConfig,
    pub command_bar: VimBar,
    pub insert_bar: VimBar,
    pub mode: AppMode,
    pub current_project: Option<String>,
    pub current_file_list: Option<ListGist>,
}

impl App {
    pub fn new() -> App {
        App {
            tasks: Vec::new(),
            size: Rect::default(),
            selected_index: 0,
            filter: AppFilterMode::All,
            client_config: Default::default(),
            command_bar: VimBar::new(),
            insert_bar: VimBar::new(),
            mode: AppMode::Global,
            current_project: None,
            current_file_list: None,
        }
    }

    pub async fn init(&mut self) {
        self.refresh_projects().await;
        self.current_project = self.client_config.current_project.to_owned();
    }

    pub async fn sync(&mut self) {
        if let (Some(list), Some(proj)) =
            (&self.current_file_list, &self.current_project)
        {
            if let Ok(gist) =
                list.get_url_gist_file(&self.client_config.client_id, &proj)
            {
                let data =
                    get_gist_file(&gist, &self.client_config.client_secret)
                        .await
                        .unwrap();
                let actual_data: Vec<WorkItem> =
                    serde_json::from_str(&data).unwrap();
                self.tasks = actual_data;
            }
        }
    }

    fn find_and_set_project(&mut self, project: &str) -> bool {
        self.get_projects().iter().find(|v| v == &project)
            .and_then(
                |x| {
                    self.current_project = Some(project.to_string());
                    Some(x)
                }
            ).is_some()
    }

    #[allow(unused_must_use)]
    pub async fn select_project(&mut self, project: &str) {
        if !self.find_and_set_project(project) {
            self.refresh_projects().await;
            self.find_and_set_project(project);
        }

        self.client_config.current_project = self.current_project.to_owned();
        self.client_config.save_config();
    }

    #[allow(unused_must_use)]
    pub async fn new_project(&mut self, project: &str) {
        self.current_project = Some(project.to_string());
        self.client_config.current_project = self.current_project.to_owned();
        self.client_config.save_config();

        self.tasks.drain(..);
        self.save_project(false, true);
    }

    pub fn save_project(&mut self, wait: bool, sync: bool) {
        if let (Some(ref proj), Some(list)) =
            (&self.current_project, &self.current_file_list)
        {
            let s = ::serde_json::to_string(&self.tasks).unwrap();
            let client_id = self.client_config.client_id.to_owned();
            let client_secret = self.client_config.client_secret.to_owned();
            let proj_copy = proj.clone();
            let list_copy = list.clone();
            let thing = async move {
                let gg = GistUpdate::new(
                    s,
                    proj_copy.to_string(),
                    proj_copy.to_string(),
                    Some(proj_copy.to_string()),
                );

                let file = list_copy.search_url_gist(&client_id).unwrap();

                gg.update(&file, &client_secret).await
            };

            if wait {
                executor::block_on(thing);
            } else {
                task::spawn(thing);
            }

            if sync {
                executor::block_on(self.init());
            }
        }
    }

    pub async fn refresh_projects(&mut self) {
        self.current_file_list = Some(
            ListGist::get_update_list_gist(&self.client_config.client_secret)
                .await
                .unwrap(),
        );
    }

    pub fn get_projects(&self) -> Vec<String> {
        if let Some(list) = &self.current_file_list {
            let gist = list.search_gist(&self.client_config.client_id).unwrap();
            return gist
                .files
                .iter()
                .map(|(_, file)| file.name.clone())
                .collect::<Vec<String>>();
        }

        Vec::new()
    }

    pub fn get_cursor_position(&self) -> u16 {
        match self.mode {
            AppMode::Global => 0,
            AppMode::Command => self.command_bar.input_cursor_position(),
            AppMode::Insert => self.insert_bar.input_cursor_position(),
        }
    }

    pub fn update_work_item_text(&mut self, id: &str, content: &str) {
        if let Some(task) =
            self.tasks.iter_mut().find(|s| s.id == Some(id.to_string()))
        {
            task.content = Some(content.to_string())
        }
    }

    pub fn get_view(&self) -> Vec<WorkItem> {
        self.tasks
            .iter()
            .filter(|l| l.is_valid_for_mode(self.filter))
            .map(|m| m)
            .cloned()
            .collect()
    }

    pub fn start_task(&mut self, id: &str) {
        if let Some(task) =
            self.tasks.iter_mut().find(|s| s.id == Some(id.to_string()))
        {
            task.start()
        }
    }

    pub fn finish_task(&mut self, id: &str) {
        if let Some(task) =
            self.tasks.iter_mut().find(|s| s.id == Some(id.to_string()))
        {
            task.finish()
        }
    }

    pub fn wont_task(&mut self, id: &str) {
        if let Some(task) =
            self.tasks.iter_mut().find(|s| s.id == Some(id.to_string()))
        {
            task.wont_fix()
        }
    }

    pub fn remove_task(&mut self, id: &str) {
        if let Some(task) = self
            .tasks
            .iter_mut()
            .position(|s| s.id == Some(id.to_string()))
        {
            self.tasks.remove(task);
        }
    }

    pub fn fix_all_work_tems(&mut self) {
        for x in self.tasks.iter_mut() {
            x.id = Some(Uuid::new_v4().to_string());
        }
    }
}
