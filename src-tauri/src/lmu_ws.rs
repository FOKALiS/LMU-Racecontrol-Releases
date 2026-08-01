//! WebSocket-Client für die LMU Replay-Metrics-Verbindung.
//!
//! WICHTIGE ERKENNTNIS: Die REST-API (replaytime, replayCommand) funktioniert
//! NUR, wenn ein WebSocket-Client auf `ws://localhost:6398/websocket/replaymetrics`
//! verbunden ist. BCUK hält diese Verbindung immer offen.
//!
//! Diese Verbindung sendet kontinuierlich `replayMetrics`-Daten
//! (currentReplayPos, lapNumber, etc.) und aktiviert die Replay-API.

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

const WS_URL: &str = "ws://localhost:6398/websocket/replaymetrics";

pub struct LmuWebSocket {
    running: Arc<AtomicBool>,
    _handle: Arc<AsyncMutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl LmuWebSocket {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            _handle: Arc::new(AsyncMutex::new(None)),
        }
    }

    /// Startet die WebSocket-Verbindung im Hintergrund.
    /// Der Task verbindet sich, hält die Verbindung offen und liest Daten.
    pub async fn start(&self) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        tokio::spawn(async move {
            while running.load(Ordering::SeqCst) {
                match Self::connect_and_read(running.clone()).await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("[lmu_ws] ⚠️ Verbindung beendet: {e:#}");
                    }
                }
                // Reconnect nach 2 Sekunden
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });

        Ok(())
    }

    /// Stoppt die WebSocket-Verbindung.
    pub async fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    async fn connect_and_read(running: Arc<AtomicBool>) -> Result<()> {
        eprintln!("[lmu_ws] 🔄 Verbinde mit {}", WS_URL);

        let (mut ws_stream, _) = connect_async(WS_URL)
            .await
            .with_context(|| format!("WebSocket-Verbindung zu {} fehlgeschlagen", WS_URL))?;

        eprintln!("[lmu_ws] ✅ WebSocket verbunden – Replay-API aktiviert!");

        // Nachrichten lesen (hält die Verbindung offen)
        while running.load(Ordering::SeqCst) {
            match ws_stream.next().await {
                Some(Ok(Message::Text(text))) => {
                    // Replay-Metrics verarbeiten (optional, nur Logging)
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(pos) = value
                            .get("body")
                            .and_then(|b| b.get("currentReplayPos"))
                            .and_then(|v| v.as_f64())
                        {
                            trace_replay_pos(pos);
                        }
                    }
                }
                Some(Ok(Message::Binary(data))) => {
                    // Binär-Nachrichten ignorieren
                    let _ = data;
                }
                Some(Ok(Message::Ping(data))) => {
                    let _ = ws_stream.send(Message::Pong(data)).await;
                }
                Some(Ok(_)) => {
                    // Andere Nachrichten ignorieren
                }
                Some(Err(e)) => {
                    return Err(anyhow::anyhow!("WebSocket-Fehler: {e}"));
                }
                None => {
                    return Err(anyhow::anyhow!("WebSocket geschlossen"));
                }
            }
        }

        Ok(())
    }
}

/// Loggt die Replay-Position (max. 1x pro Sekunde, um Log-Flut zu vermeiden).
fn trace_replay_pos(pos: f64) {
    use std::sync::Mutex;
    use std::time::Instant;
    use once_cell::sync::Lazy;

    static LAST_LOG: Lazy<Mutex<Option<Instant>>> = Lazy::new(|| Mutex::new(None));
    let mut last = LAST_LOG.lock().unwrap();
    if last.map(|t| t.elapsed().as_secs() >= 5).unwrap_or(true) {
        eprintln!("[lmu_ws] 📊 Replay-Position: {:.1}s", pos);
        *last = Some(Instant::now());
    }
}