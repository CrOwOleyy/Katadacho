mod ping;
mod transfer;

use std::time::Duration;

use serde::Serialize;
use tauri::ipc::Channel;

const PING_URL: &str = "https://speed.cloudflare.com/__down?bytes=0";
const PING_SAMPLES: usize = 10;
const DOWNLOAD_DURATION: Duration = Duration::from_secs(6);
const UPLOAD_DURATION: Duration = Duration::from_secs(5);

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "stage", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum SpeedtestEvent {
    Ping { latency_ms: f64, jitter_ms: f64 },
    Download { mbps: f64, progress: f32 },
    Upload { mbps: f64, progress: f32 },
    Done { download_mbps: f64, upload_mbps: f64 },
}

/// Speedtest complet : latence et gigue, puis téléchargement de 6 s, puis envoi de 5 s.
/// Chaque étape est envoyée au frontend via `channel` au fil de l'eau.
#[tauri::command]
pub async fn run_speedtest(channel: Channel<SpeedtestEvent>) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent("Katadacho/0.1")
        .build()
        .map_err(|e| e.to_string())?;

    let ping = ping::measure(&client, PING_URL, PING_SAMPLES).await?;
    channel
        .send(SpeedtestEvent::Ping {
            latency_ms: ping.latency_ms,
            jitter_ms: ping.jitter_ms,
        })
        .map_err(|e| e.to_string())?;

    let download_mbps = transfer::download(&client, &channel, DOWNLOAD_DURATION).await?;
    let upload_mbps = transfer::upload(&client, &channel, UPLOAD_DURATION).await?;

    channel
        .send(SpeedtestEvent::Done {
            download_mbps,
            upload_mbps,
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}
