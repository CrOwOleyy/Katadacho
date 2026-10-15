use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bytes::Bytes;
use futures_util::{stream, StreamExt};
use tauri::ipc::Channel;

use super::SpeedtestEvent;

const CONNECTIONS: usize = 4;
const SAMPLE_INTERVAL: Duration = Duration::from_millis(150);
const UPLOAD_CHUNK_BYTES: usize = 64 * 1024;
pub const DOWNLOAD_URL: &str = "https://speed.cloudflare.com/__down?bytes=50000000";
pub const UPLOAD_URL: &str = "https://speed.cloudflare.com/__up";

/// Mégaoctets par seconde converti en Mbps, sur une fenêtre de `seconds`.
fn to_mbps(bytes: u64, seconds: f64) -> f64 {
    bytes as f64 * 8.0 / seconds.max(0.001) / 1_000_000.0
}

/// Lance `CONNECTIONS` téléchargements en parallèle pendant `duration`, et envoie
/// le débit instantané toutes les 150 ms. Retourne le débit moyen final en Mbps.
pub async fn download(
    client: &reqwest::Client,
    channel: &Channel<SpeedtestEvent>,
    duration: Duration,
) -> Result<f64, String> {
    let counter = Arc::new(AtomicU64::new(0));
    let started = Instant::now();
    let deadline = started + duration;

    let workers: Vec<_> = (0..CONNECTIONS)
        .map(|_| {
            let client = client.clone();
            let counter = Arc::clone(&counter);
            tokio::spawn(async move {
                while Instant::now() < deadline {
                    let Ok(response) = client.get(DOWNLOAD_URL).send().await else {
                        break;
                    };
                    let mut body = response.bytes_stream();
                    while let Some(chunk) = body.next().await {
                        let Ok(chunk) = chunk else { break };
                        counter.fetch_add(chunk.len() as u64, Ordering::Relaxed);
                        if Instant::now() >= deadline {
                            break;
                        }
                    }
                }
            })
        })
        .collect();

    let mut last_bytes = 0u64;
    let mut last_tick = started;
    while Instant::now() < deadline {
        tokio::time::sleep(SAMPLE_INTERVAL).await;
        let now = Instant::now();
        let total = counter.load(Ordering::Relaxed);
        let mbps = to_mbps(total - last_bytes, now.duration_since(last_tick).as_secs_f64());
        let progress = (now.duration_since(started).as_secs_f64() / duration.as_secs_f64()).min(1.0);
        channel
            .send(SpeedtestEvent::Download { mbps, progress: progress as f32 })
            .map_err(|e| e.to_string())?;
        last_bytes = total;
        last_tick = now;
    }

    for worker in workers {
        worker.abort();
    }

    Ok(to_mbps(counter.load(Ordering::Relaxed), started.elapsed().as_secs_f64()))
}

/// Envoie des buffers en mémoire vers Cloudflare pendant `duration`, sur `CONNECTIONS` flux.
/// Retourne le débit moyen final en Mbps.
pub async fn upload(
    client: &reqwest::Client,
    channel: &Channel<SpeedtestEvent>,
    duration: Duration,
) -> Result<f64, String> {
    let counter = Arc::new(AtomicU64::new(0));
    let started = Instant::now();
    let deadline = started + duration;
    let chunk = Bytes::from(vec![0u8; UPLOAD_CHUNK_BYTES]);

    let workers: Vec<_> = (0..CONNECTIONS)
        .map(|_| {
            let client = client.clone();
            let counter = Arc::clone(&counter);
            let chunk = chunk.clone();
            tokio::spawn(async move {
                let body = stream::unfold((), move |()| {
                    let counter = Arc::clone(&counter);
                    let chunk = chunk.clone();
                    async move {
                        if Instant::now() >= deadline {
                            return None;
                        }
                        counter.fetch_add(chunk.len() as u64, Ordering::Relaxed);
                        Some((Ok::<Bytes, std::io::Error>(chunk), ()))
                    }
                });
                let _ = client
                    .post(UPLOAD_URL)
                    .body(reqwest::Body::wrap_stream(body))
                    .send()
                    .await;
            })
        })
        .collect();

    let mut last_bytes = 0u64;
    let mut last_tick = started;
    while Instant::now() < deadline {
        tokio::time::sleep(SAMPLE_INTERVAL).await;
        let now = Instant::now();
        let total = counter.load(Ordering::Relaxed);
        let mbps = to_mbps(total - last_bytes, now.duration_since(last_tick).as_secs_f64());
        let progress = (now.duration_since(started).as_secs_f64() / duration.as_secs_f64()).min(1.0);
        channel
            .send(SpeedtestEvent::Upload { mbps, progress: progress as f32 })
            .map_err(|e| e.to_string())?;
        last_bytes = total;
        last_tick = now;
    }

    for worker in workers {
        worker.abort();
    }

    Ok(to_mbps(counter.load(Ordering::Relaxed), started.elapsed().as_secs_f64()))
}
