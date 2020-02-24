use anyhow::anyhow;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use yukikaze::client::{Client, Request};
use yukikaze::matsu;

pub const URL: &str = "https://api.github.com/gists";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GistUpdate {
    pub description: Option<String>,
    pub files: HashMap<String, FileUpdate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileUpdate {
    pub content: String,
    pub filename: Option<String>,
}

impl GistUpdate {
    pub fn new(
        cont: String,
        desc: String,
        old_name: String,
        new_name: Option<String>,
    ) -> Self {
        let mut hm: HashMap<String, FileUpdate> = HashMap::new();
        hm.insert(
            old_name,
            FileUpdate {
                content: cont,
                filename: new_name,
            },
        );
        GistUpdate {
            description: Some(desc),
            files: hm,
        }
    }
    pub async fn update(&self, url: &str, token: &str) -> Result<()> {
        let client = Client::default();
        let mut resp: Request =
            Request::put(url)?.bearer_auth(token).json(self)?;

        *resp.method_mut() = http::Method::PATCH;

        let mut response = matsu!(client.send(resp))
            .expect("Not timedout")
            .expect("Successful");
        matsu!(response.text()).expect("To read HTML");
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GistPost {
    pub description: String,
    pub public: bool,
    pub files: HashMap<String, FilePost>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilePost {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseGist {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "description")]
    pub desc: Option<String>,
    #[serde(rename = "files")]
    pub files: HashMap<String, FileGist>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileGist {
    #[serde(rename = "filename")]
    pub name: String,
    #[serde(rename = "type")]
    pub type_file: Option<String>,
    #[serde(rename = "language")]
    pub lang: Option<String>,
    #[serde(rename = "raw_url")]
    pub raw_url: String,
    #[serde(rename = "size")]
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct ListGist {
    pub list: Vec<ResponseGist>,
}

pub async fn get_gist_file(url: &str, token: &str) -> Result<String> {
    let client = Client::default();
    let resp: Request = Request::get(url)?.bearer_auth(token).empty();

    let mut response = matsu!(client.send(resp))
        .expect("Not timedout")
        .expect("Successful");
    if response.is_success() {
        let buf = matsu!(response.text()).expect("To read HTML");
        return Ok(buf);
    }
    Err(anyhow!("Failed to get list"))
}

impl ListGist {
    fn new(list: Vec<ResponseGist>) -> ListGist {
        ListGist { list }
    }

    pub async fn get_update_list_gist(token: &str) -> Result<ListGist> {
        let client = Client::default();
        let resp: Request = Request::get(URL)?.bearer_auth(token).empty();
        let mut response = matsu!(client.send(resp))
            .expect("Not timedout")
            .expect("Successful");
        if response.is_success() {
            let list_gist: Vec<ResponseGist> =
                matsu!(response.json()).expect("To read HTML");
            return Ok(ListGist::new(list_gist));
        }
        Err(anyhow!("unsuccessful get list gist"))
    }

    pub fn search_url_gist<T: AsRef<str>>(&self, id: T) -> Result<String> {
        if id.as_ref().len() < 5 {
            return Err(anyhow!("id invalid"));
        }
        for gist in self.list.clone() {
            if gist.id.starts_with(id.as_ref()) {
                return Ok(gist.url);
            }
        }
        Err(anyhow!("gist file not exist"))
    }

    pub fn search_gist<T: AsRef<str>>(&self, id: T) -> Result<ResponseGist> {
        if id.as_ref().len() < 5 {
            return Err(anyhow!("id invalid"));
        }
        for gist in self.list.clone() {
            if gist.id.starts_with(id.as_ref()) {
                return Ok(gist);
            }
        }
        Err(anyhow!("gist file not exist"))
    }

    pub fn get_url_gist_file<T: AsRef<str>>(
        &self,
        id: T,
        file: T,
    ) -> Result<String> {
        for gist in self.list.clone() {
            if gist.id == id.as_ref() {
                for (_, v) in gist.files {
                    if v.name == file.as_ref() {
                        return Ok(v.raw_url);
                    }
                }
            }
        }
        Err(anyhow!("id not exist"))
    }
}
