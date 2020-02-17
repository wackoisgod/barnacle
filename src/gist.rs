use reqwest::{Client, Response};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use std::io::prelude::*;

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
}

use errors::*;

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
    pub fn new(cont: String, desc: String, old_name: String, new_name: Option<String>) -> Self {
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
    pub fn update(&self, url: &str, token: &str) -> Result<String> {
        let _resp: Response = Client::new()
            .patch(url)
            .bearer_auth(token)
            .json(self)
            .send()
            .chain_err(|| "update gist faild")?;
        //let gist_spot: GistPost = resp.json().chain_err(|| "convert to GistPost faild")?;
        Ok("gist_spot".to_string())
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

// impl GistPost {
//     pub fn new(cont: String, public: bool, desc: String, name: String) -> Self {
//         let mut hm: HashMap<String, FilePost> = HashMap::new();
//         hm.insert(name, FilePost { content: cont });
//         GistPost {
//             description: desc,
//             public: public,
//             files: hm,
//         }
//     }
//     pub fn post(&self) -> Result<GistPost> {
//         let mut resp: Response = Client::new()
//             .post(URL)
//             .bearer_auth(&*TOKEN)
//             .json(self)
//             .send()
//             .chain_err(|| "post gist unsuccess !")?;
//         let rs_data: GistPost = resp.json().chain_err(|| "convert to json error")?;
//         Ok(rs_data)
//     }
// }

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

pub struct ListGist {
    pub list: Vec<ResponseGist>,
}

pub fn get_gist_file(url: &str, token: &str) -> Result<String> {
    let mut resp: Response = Client::new()
        .get(url)
        .bearer_auth(token)
        .send()
        .chain_err(|| format!("failed get gist file {}", url))?;
    if resp.status().is_success() {
        let mut buf = String::new();
        resp.read_to_string(&mut buf).unwrap();
        return Ok(buf);
    }
    Err(Error::from("unsuccessful get gist file"))
}

impl ListGist {
    fn new(list: Vec<ResponseGist>) -> ListGist {
        ListGist { list }
    }

    // pub fn read() -> Result<ListGist> {
    //     let path_file = &utils::path_file_in_home(LIST_GIST_FILE_NAME).unwrap();
    //     let out = utils::read_file(path_file).unwrap();
    //     let gists: Vec<ResponseGist> = serde_json::from_str(&out)
    //         .chain_err(|| format!("failed read file {}", path_file.to_str().unwrap()))?;
    //     return Ok(ListGist::new(gists));
    // }

    // fn write(&self) -> Result<()> {
    //     let path_file = utils::path_file_in_home(LIST_GIST_FILE_NAME).unwrap();
    //     let list_string = serde_json::to_string(&self.list).chain_err(|| "can't get list")?;
    //     return utils::write_file(path_file, list_string);
    // }

    pub fn get_update_list_gist(token: &str) -> Result<ListGist> {
        let mut resp: Response = Client::new()
            .get(URL)
            .bearer_auth(token)
            .send()
            .chain_err(|| "failed get list")?;
        if resp.status().is_success() {
            let list_gist: Vec<ResponseGist> = resp.json().chain_err(|| "can't read gist list")?;
            return Ok(ListGist::new(list_gist));
        }
        Err(Error::from("unsuccessful get list gist"))
    }

    pub fn search_url_gist<T: AsRef<str>>(&self, id: T) -> Result<String> {
        if id.as_ref().len() < 5 {
            return Err(Error::from("id invalid"));
        }
        for gist in self.list.clone() {
            if gist.id.starts_with(id.as_ref()) {
                return Ok(gist.url);
            }
        }
        Err(Error::from("gist file not exist"))
    }

    pub fn search_gist<T: AsRef<str>>(&self, id: T) -> Result<ResponseGist> {
        if id.as_ref().len() < 5 {
            return Err(Error::from("id invalid"));
        }
        for gist in self.list.clone() {
            if gist.id.starts_with(id.as_ref()) {
                return Ok(gist);
            }
        }
        Err(Error::from("gist file not exist"))
    }

    pub fn get_url_gist_file<T: AsRef<str>>(&self, id: T, file: T) -> Result<String> {
        for gist in self.list.clone() {
            if gist.id == id.as_ref() {
                for (_, v) in gist.files {
                    if v.name == file.as_ref() {
                        return Ok(v.raw_url);
                    }
                }
            }
        }
        Err(Error::from("id not exist"))
    }
}
