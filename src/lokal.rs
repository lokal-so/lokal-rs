use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Lokal {
    pub base_url: String,
    pub basic_auth: (String, String),
    pub token: String,
    #[serde(skip)]
    pub rest: Client,
}

impl Lokal {
    pub fn new_default() -> Self {
        let rest = Client::new();
        Lokal {
            base_url: "http://127.0.0.1:6174".to_string(),
            basic_auth: (String::new(), String::new()),
            token: String::new(),
            rest,
        }
    }

    pub fn set_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    pub fn set_basic_auth(mut self, username: String, password: String) -> Self {
        self.basic_auth = (username, password);
        self
    }

    pub fn set_api_token(mut self, token: String) -> Self {
        self.token = token;
        self
    }
}
