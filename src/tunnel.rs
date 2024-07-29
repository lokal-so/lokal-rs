use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::error::Error;
use rand::Rng;
use colored::Colorize;

use crate::lokal::Lokal;

#[derive(Debug, Serialize, Deserialize)]
pub enum TunnelType {
    HTTP,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Options {
    pub basic_auth: Vec<String>,
    pub cidr_allow: Vec<String>,
    pub cidr_deny: Vec<String>,
    pub request_header_add: Vec<String>,
    pub request_header_remove: Vec<String>,
    pub response_header_add: Vec<String>,
    pub response_header_remove: Vec<String>,
    pub header_key: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tunnel {
    #[serde(skip)]
    pub lokal: Option<Lokal>,
    #[serde(skip)]
    pub id: Option<String>,
    pub name: String,
    pub tunnel_type: TunnelType,
    pub local_address: String,
    pub server_id: String,
    pub address_tunnel: String,
    pub address_tunnel_port: i64,
    pub address_public: String,
    pub address_mdns: String,
    pub inspect: bool,
    pub options: Options,
    pub ignore_duplicate: bool,
    pub startup_banner: bool,
}

impl Tunnel {
    pub fn new(lokal: Lokal) -> Self {
        Tunnel {
            lokal: Some(lokal),
            id: None,
            name: String::new(),
            tunnel_type: TunnelType::HTTP,
            local_address: String::new(),
            server_id: String::new(),
            address_tunnel: String::new(),
            address_tunnel_port: 0,
            address_public: String::new(),
            address_mdns: String::new(),
            inspect: false,
            options: Options {
                basic_auth: vec![],
                cidr_allow: vec![],
                cidr_deny: vec![],
                request_header_add: vec![],
                request_header_remove: vec![],
                response_header_add: vec![],
                response_header_remove: vec![],
                header_key: vec![],
            },
            ignore_duplicate: false,
            startup_banner: false,
        }
    }

    pub fn set_local_address(mut self, local_address: String) -> Self {
        self.local_address = local_address;
        self
    }

    pub fn set_tunnel_type(mut self, tunnel_type: TunnelType) -> Self {
        self.tunnel_type = tunnel_type;
        self
    }

    pub fn set_inspection(mut self, inspect: bool) -> Self {
        self.inspect = inspect;
        self
    }

    pub fn set_lan_address(mut self, lan_address: String) -> Self {
        self.address_mdns = lan_address.trim_end_matches(".local").to_string();
        self
    }

    pub fn set_public_address(mut self, public_address: String) -> Self {
        self.address_public = public_address;
        self
    }

    pub fn set_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn ignore_duplicate(mut self) -> Self {
        self.ignore_duplicate = true;
        self
    }

    pub fn show_startup_banner(mut self) -> Self {
        self.startup_banner = true;
        self
    }

    pub async fn create(&mut self) -> Result<(), Box<dyn Error>> {
        if self.address_mdns.is_empty() && self.address_public.is_empty() {
            return Err("please enable either lan address or random/custom public url".into());
        }

        let client = Client::new();
        let resp = client
            .post(format!("{}/api/tunnel/start", self.lokal.as_ref().unwrap().base_url))
            .json(&self)
            .send()
            .await?
            .json::<Response>()
            .await?;

        if resp.tunnel.is_empty() {
            return Err("tunnel creation failing".into());
        }

        if !resp.success {
            if self.ignore_duplicate && resp.message.ends_with("address is already being used") {
                self.address_public = resp.tunnel[0].address_public.clone();
                self.address_mdns = resp.tunnel[0].address_mdns.clone();
                self.id = Some(resp.tunnel[0].id.clone());

                self.show_banner().await;
                return Ok(());
            }
            return Err(resp.message.into());
        }

        self.address_public = resp.tunnel[0].address_public.clone();
        self.address_mdns = resp.tunnel[0].address_mdns.clone();
        self.id = Some(resp.tunnel[0].id.clone());

        self.show_banner().await;

        Ok(())
    }

    pub async fn get_lan_address(&self) -> Result<String, Box<dyn Error>> {
        if self.address_mdns.is_empty() {
            return Err("lan address is not being set".into());
        }

        if !self.address_mdns.ends_with(".local") {
            return Ok(format!("{}.local", self.address_mdns));
        }

        Ok(self.address_mdns.clone())
    }

    pub async fn get_public_address(&mut self) -> Result<String, Box<dyn Error>> {
        if self.address_public.is_empty() {
            return Err("public address is not requested by client".into());
        }

        if !self.address_public.contains(':') {
            self.update_public_url_port().await?;
            return Err("tunnel is using a random port, but it has not been assigned yet. please try again later".into());
        }

        Ok(self.address_public.clone())
    }

    pub async fn update_public_url_port(&mut self) -> Result<(), Box<dyn Error>> {
        let client = Client::new();
        let resp = client
            .get(&format!("{}/api/tunnel/info/{}", self.lokal.as_ref().unwrap().base_url, self.id.as_ref().unwrap()))
            .send()
            .await?
            .json::<Response>()
            .await?;

        if !resp.success {
            return Err(resp.message.into());
        }

        if resp.tunnel.is_empty() {
            return Err("could not get tunnel info".into());
        }

        if !resp.tunnel[0].address_public.contains(':') {
            return Err("could not get assigned port".into());
        }

        self.address_public = resp.tunnel[0].address_public.clone();

        Ok(())
    }

    pub async fn show_banner(&mut self) {
        if !self.startup_banner {
            return;
        }

        let banner = r#"
    __       _         _             
   / /  ___ | | ____ _| |  ___  ___  
  / /  / _ \| |/ / _  | | / __|/ _ \ 
 / /__| (_) |   < (_| | |_\__ \ (_) |
 \____/\___/|_|\_\__,_|_(_)___/\___/ "#;

        let colors = [
            |text: &str| text.magenta().to_string(),
            |text: &str| text.blue().to_string(),
            |text: &str| text.cyan().to_string(),
            |text: &str| text.green().to_string(),
            |text: &str| text.red().to_string(),
        ];

        let mut rng = rand::thread_rng();
        let color = colors[rng.gen_range(0..colors.len())];
        println!("{}", color(banner));
        println!();
        println!("{}", "Minimum Lokal Client".red());
        if let Ok(val) = self.get_public_address().await {
            println!("{}", format!("Public Address \t\thttps://{}", val).cyan());
        }
        if let Ok(val) = self.get_lan_address().await {
            println!("{}", format!("LAN Address \t\thttps://{}", val).green());
        }
        println!();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    success: bool,
    message: String,
    #[serde(rename = "data")]
    tunnel: Vec<TunnelResponse>,
}

#[derive(Deserialize, Debug, Serialize)]
struct TunnelResponse {
    address_public: String,
    address_mdns: String,
    id: String,
}