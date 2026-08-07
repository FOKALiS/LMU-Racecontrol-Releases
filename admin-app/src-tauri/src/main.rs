// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    data: Option<String>,
    error: Option<String>,
}

#[tauri::command]
async fn api_request(url: String, method: String, body: Option<String>) -> ApiResponse {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let req = match method.as_str() {
        "POST" => {
            let mut r = client.post(&url);
            if let Some(b) = &body {
                r = r.header("Content-Type", "application/json").body(b.clone());
            }
            r
        }
        "PATCH" => {
            let mut r = client.patch(&url);
            if let Some(b) = &body {
                r = r.header("Content-Type", "application/json").body(b.clone());
            }
            r
        }
        _ => client.get(&url),
    };

    match req.send().await {
        Ok(resp) => {
            let text = resp.text().await.unwrap_or_default();
            ApiResponse { success: true, data: Some(text), error: None }
        }
        Err(e) => {
            ApiResponse { success: false, data: None, error: Some(e.to_string()) }
        }
    }
}

#[tauri::command]
async fn api_request_with_key(url: String, method: String, api_key: String, body: Option<String>) -> ApiResponse {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let mut req = match method.as_str() {
        "POST" => {
            let mut r = client.post(&url);
            if let Some(b) = &body {
                r = r.header("Content-Type", "application/json").body(b.clone());
            }
            r
        }
        "PATCH" => {
            let mut r = client.patch(&url);
            if let Some(b) = &body {
                r = r.header("Content-Type", "application/json").body(b.clone());
            }
            r
        }
        "DELETE" => client.delete(&url),
        _ => client.get(&url),
    };

    req = req.header("Authorization", format!("Bearer {}", api_key));

    match req.send().await {
        Ok(resp) => {
            let text = resp.text().await.unwrap_or_default();
            ApiResponse { success: true, data: Some(text), error: None }
        }
        Err(e) => {
            ApiResponse { success: false, data: None, error: Some(e.to_string()) }
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![api_request, api_request_with_key])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}