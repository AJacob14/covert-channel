use base64::{engine::general_purpose, Engine};
use clap::{command, Parser};
use reqwest::header::{HeaderMap, HeaderValue};
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg()]
    filepath: String,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let endpoint = "http://192.168.56.102:3000";
    let start_enpoint = format!("{}/embeddings", endpoint);
    let transmit_endpoint = format!("{}/chat/completions", endpoint);
    let end_endpoint = format!("{}/batches", endpoint);

    let file = args.filepath;
    let result = fs::read(file);
    if result.is_err(){
        eprint!("Failed to read file\nReason: {}", result.unwrap_err());
        return;
    }

    let data = result.unwrap();
    let data = general_purpose::STANDARD.encode(data);
    let data = chunk_str(&data, 200);
    let client = reqwest::Client::new();

    let response = client.post(start_enpoint)
        .send()
        .await;

    if response.is_err() {
        eprintln!("HTTP Request failed\nReason: {}", response.unwrap_err());
        return;
    }

    for part in data {
        let mut headers = HeaderMap::new();
        let auth = format!("Bearer {}", part);
        let header_value = HeaderValue::from_str(auth.as_str());
        if header_value.is_err() {
            eprintln!("Failed to construct header\nReason: {}", header_value.unwrap_err());
            return;
        }

        let header_value = header_value.unwrap();
        headers.insert("Authorization", header_value);

        let response = client.post(&transmit_endpoint)
            .headers(headers)
            .send()
            .await;

        if response.is_err() {
            eprintln!("HTTP Request failed\nReason: {}", response.unwrap_err());
            return;
        }
    }

    let response = client.post(end_endpoint)
        .send()
        .await;

    if response.is_err() {
        eprintln!("HTTP Request failed\nReason: {}", response.unwrap_err());
        return;
    }
}

fn chunk_str(string: &String, chunk_size: usize) -> Vec<String> {
    let mut result = Vec::new();
    
    let mut start = 0;
    while start < string.len() {
        let end = start + chunk_size;
        let chunk = if end < string.len() {
            &string[start..end]
        }
        else{
            &string[start..]
        };
        start = end;
        result.push(chunk.to_string());
    }

    result
}