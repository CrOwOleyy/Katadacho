use std::time::Instant;

pub struct PingResult {
    pub latency_ms: f64,
    pub jitter_ms: f64,
}

/// Mesure la latence moyenne et la gigue (moyenne des écarts entre mesures successives)
/// à partir de requêtes HEAD, qui ne transfèrent aucun corps.
pub async fn measure(client: &reqwest::Client, url: &str, samples: usize) -> Result<PingResult, String> {
    let mut times: Vec<f64> = Vec::with_capacity(samples);

    for _ in 0..samples {
        let started = Instant::now();
        client
            .head(url)
            .send()
            .await
            .and_then(|response| response.error_for_status())
            .map_err(|e| e.to_string())?;
        times.push(started.elapsed().as_secs_f64() * 1000.0);
    }

    let latency_ms = mean(&times);
    let jitter_ms = if times.len() < 2 {
        0.0
    } else {
        let deltas: Vec<f64> = times.windows(2).map(|pair| (pair[1] - pair[0]).abs()).collect();
        mean(&deltas)
    };

    Ok(PingResult { latency_ms, jitter_ms })
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}
