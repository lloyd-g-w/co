//! The co desktop app: the web frontend in a native window, plus an
//! in-process tool host so this machine can run tools for the user's
//! sessions while the app is open.

use std::time::Duration;

use tauri::Manager;

const DEFAULT_SERVER_URL: &str = "ws://localhost:7420/ws";

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    tauri::Builder::default()
        // Must be first: a second launch focuses the running app instead.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            tauri::async_runtime::spawn(tool_host_loop());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the co desktop app");
}

/// Keeps this machine connected as a tool host, reconnecting with backoff.
async fn tool_host_loop() {
    let server_url =
        std::env::var("CO_SERVER_URL").unwrap_or_else(|_| DEFAULT_SERVER_URL.to_string());
    let name = machine_name();
    let mut delay = Duration::from_millis(500);
    loop {
        let config = co_tool_host::Config {
            server_url: server_url.clone(),
            name: name.clone(),
            host_id: None,
        };
        match co_tool_host::run(config).await {
            Ok(()) => {
                tracing::info!("tool host connection closed");
                delay = Duration::from_millis(500);
            }
            Err(err) => tracing::warn!(error = %err, "tool host connection failed"),
        }
        tokio::time::sleep(delay).await;
        delay = (delay * 2).min(Duration::from_secs(30));
    }
}

fn machine_name() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "desktop".to_string())
}
