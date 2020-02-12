use reqwest::{Client, Response};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use lazy_static::lazy_static;
use std::fs::File;
use std::io::prelude::*;

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
}

use errors::*;

lazy_static! {
    pub static ref TOKEN: String = String::from(Token::new("1647f2b78fa25ea3deb1cbd7ee314bfabbe5e135".to_string()).get());
}

pub const TOKEN_FILE_NAME:&'static str = ".gist-rs";
pub const LIST_GIST_FILE_NAME:&'static str = ".list-gist";
pub const URL: &'static str = "https://api.github.com/gists";


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
        pub fn update(&self, url: &str) -> Result<GistPost> {
                let mut resp: Response = Client::new()
                        .patch(url)
                        .bearer_auth(TOKEN.clone())
                        .json(self)
                        .send()
                        .chain_err(|| "update gist faild")?;
                println!("{}", resp.text().unwrap());
                let gist_spot: GistPost = resp.json().chain_err(|| "convert to GistPost faild")?;
                Ok(gist_spot)
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

impl GistPost {
    pub fn new(cont: String, public: bool, desc: String, name: String) -> Self {
        let mut hm: HashMap<String, FilePost> = HashMap::new();
        hm.insert(name, FilePost { content: cont });
        GistPost {
            description: desc,
            public: public,
            files: hm,
        }
    }
    pub fn post(&self) -> Result<GistPost> {
        let mut resp: Response = Client::new()
            .post(URL)
            .bearer_auth(&*TOKEN)
            .json(self)
            .send()
            .chain_err(|| "post gist unsuccess !")?;
        let rs_data: GistPost = resp.json().chain_err(|| "convert to json error")?;
        Ok(rs_data)
    }
}

pub struct Token(String);

impl Token {
    fn new(token: String) -> Token {
        return Token(token);
    }

    pub fn get(&self) -> String {
        return self.0.clone();
    }

    // pub fn read() -> Result<Self> {
    //     let path_file = utils::path_file_in_home(TOKEN_FILE_NAME).unwrap();
    //     let mut file = File::open(path_file.as_path())
    //         .chain_err(|| format!("failed open file {}", path_file.to_str().unwrap()))?;
    //     let mut token = String::new();
    //     file.read_to_string(&mut token)
    //         .chain_err(|| format!("failed read file {}", path_file.to_str().unwrap()))?;
    //     return Ok(Token::new(token));
    // }

    // pub fn write(token: String) -> Result<()> {
    //     let path_file = utils::path_file_in_home(TOKEN_FILE_NAME).unwrap();
    //     let mut file = File::create(path_file.as_path())
    //         .chain_err(|| format!("failed create file {}", path_file.to_str().unwrap()))?;
    //     file.write(token.as_bytes())
    //         .chain_err(|| format!("failed write file {}", path_file.to_str().unwrap()))?;
    //     Ok(())
    // }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ResponseGist {
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
struct FileGist {
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
    list: Vec<ResponseGist>,
}

pub fn get_gist_file(url: &str) -> Result<String> {
    let mut resp: Response = Client::new()
        .get(url)
        .bearer_auth(&*TOKEN)
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
        return ListGist { list };
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

    pub fn get_update_list_gist() -> Result<ListGist> {
        let mut resp: Response = Client::new()
            .get(URL)
            .bearer_auth(&*TOKEN)
            .send()
            .chain_err(|| "failed get list")?;
        if resp.status().is_success() {
            let list_gist: Vec<ResponseGist> = resp.json().chain_err(|| "can't read gist list")?;
            return Ok(ListGist::new(list_gist));
        }
        return Err(Error::from("unsuccessful get list gist"));
    }

    // pub fn sync() -> Result<ListGist> {
    //     let list_gist = ListGist::get_update_list_gist().unwrap();
    //     list_gist.write().unwrap();        
    //     Ok(list_gist)
    // }

    pub fn _search_url_gist<T: AsRef<str>>(&self, id: T) -> Result<String> {
        if id.as_ref().len() < 5 {
            return Err(Error::from("id invalid"));
        }
        for gist in self.list.clone() {
            if gist.id.starts_with(id.as_ref()) {
                return Ok(gist.url);
            }
        }
        return Err(Error::from("gist file not exist"));
    }

    // pub fn search_raw_url_gist<T: AsRef<str>>(&self, id: T) -> Result<String> {
    //     if id.as_ref().len() < 5 {
    //         return Err(Error::from("id len most be bigger than 5"));
    //     }
    //     for gist in self.list.clone() {
    //         if gist.id.starts_with(id.as_ref()) {
    //             for (_, v) in gist.files {
    //                 return Ok(v.raw_url);
    //             }
    //         }
    //     }
    //     return Err(Error::from("gist not exist"));
    // }

    pub fn get_name_gist_file<T: AsRef<str>>(&self, id: T) -> Result<String> {
        for gist in self.list.clone() {
            if gist.id == id.as_ref() {
                for (_, v) in gist.files {
                    return Ok(v.name);
                }
            }
        }
        return Err(Error::from("id not exist"));
    }

    pub fn get_url_gist_file<T: AsRef<str>>(&self, id: T) -> Result<String> {
        for gist in self.list.clone() {
            if gist.id == id.as_ref() {
                for (_, v) in gist.files {
                    return Ok(v.raw_url);
                }
            }
        }
        return Err(Error::from("id not exist"));
    }

    pub fn print(&self, verbose: bool) -> Result<()> {
        let mut count = 0;
        for gist in self.list.clone() {
            println!(
                "{}) {}",
                count,
                gist.desc.unwrap_or("None Description".to_owned()),
            );
            println!("{}", gist.id);
            if verbose {
                for (_, v) in gist.files {
                    println!("{}", v.raw_url);
                    println!("---------------------------------------------------------");
                }
            }
            count += 1;
        }
        Ok(())
    }
}