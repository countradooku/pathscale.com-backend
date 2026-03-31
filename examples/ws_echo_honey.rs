//! Connect to a WebSocket server using HoneyIdConnection — same code path as the SendMsg handler.
//! Sends raw stdin lines as the `params` field and prints whatever the server echoes back.
//!
//! Usage: cargo run --example ws_echo_honey -- <server_url> [protocol]
//! e.g.:  cargo run --example ws_echo_honey -- wss://ws-echo.liftmap.pro

use std::io::{self, BufRead};

use honey_id_types::HoneyIdConnection;
use rustls::crypto::ring;
use serde_json::Value;
use url::Url;

const METHOD_ID: u32 = 1;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    ring::default_provider()
        .install_default()
        .expect("Could not install default crypto provider");

    let mut args = std::env::args().skip(1);
    let server = args.next().expect("Usage: ws_echo_honey <server_url> [protocol]");
    let protocol = args.next();

    let url = Url::parse(&server)?;

    println!("Connecting to {url} via HoneyIdConnection...");
    let mut conn = HoneyIdConnection::connect(&url, protocol.as_deref())
        .await
        .map_err(|e| eyre::eyre!("{e}"))?;
    println!("Connected. Type a message and press Enter.\n");

    for line in io::stdin().lock().lines() {
        let message = line?;
        if message.is_empty() {
            continue;
        }
        println!("-> {message}");
        conn.send_request_raw(METHOD_ID, &message).await?;

        let response: Value = conn.receive_response().await?;
        println!("<- {response}");
    }

    Ok(())
}
