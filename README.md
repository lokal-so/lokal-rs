# Lokal Rust

Rust crates for interacting with Lokal Client REST API

```rust
use lokal_rs::lokal::Lokal;
use lokal_rs::tunnel::{Tunnel, TunnelType};
use tokio;

#[tokio::main]
async fn main() {
    let lokal = Lokal::new_default()
        .set_base_url("http://localhost:6174".to_string())
        .set_basic_auth("username".to_string(), "password".to_string())
        .set_api_token("your_api_token".to_string());

    let mut tunnel = Tunnel::new(lokal)
        .set_local_address("127.0.0.1:8080".to_string())
        .set_tunnel_type(TunnelType::HTTP)
        .set_inspection(true)
        .set_lan_address("axum-backend.local".to_string())
        .set_public_address("axum.k.lokal-so.site".to_string())
        .set_name("My Tunnel".to_string())
        .ignore_duplicate()
        .show_startup_banner();
    
    match tunnel.create().await {
        Ok(_) => println!("Tunnel created successfully!"),
        Err(e) => println!("Error creating tunnel: {}", e),
    }
}
```
