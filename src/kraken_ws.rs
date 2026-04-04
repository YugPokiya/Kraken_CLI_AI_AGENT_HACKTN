use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use tokio_tungstenite::connect_async;
use tokio_util::sync::CancellationToken;

use crate::configuration::Settings;
use crate::state::SharedMarketState;

#[derive(Debug, Deserialize)]
struct TickerData {
    last: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WsEnvelope {
    channel: Option<String>,
    data: Option<Vec<TickerData>>,
}

pub async fn run_market_data_ingest(
    state: SharedMarketState,
    settings: Settings,
    shutdown: CancellationToken,
) -> Result<()> {
    let (mut ws, _) = connect_async(&settings.ws_url).await?;

    let subscribe_msg = json!({
        "method": "subscribe",
        "params": {
            "channel": "ticker",
            "symbol": [state.read().await.pair.clone()],
        }
    });

    ws.send(tokio_tungstenite::tungstenite::Message::Text(
        subscribe_msg.to_string().into(),
    ))
    .await?;

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("WebSocket ingest received shutdown signal");
                break;
            }
            msg = ws.next() => {
                let Some(msg) = msg else { break };
                let msg = msg?;
                if let tokio_tungstenite::tungstenite::Message::Text(payload) = msg {
                    handle_payload(&payload, &state, settings.window_size).await;
                }
            }
        }
    }

    Ok(())
}

async fn handle_payload(payload: &str, state: &SharedMarketState, window_size: usize) {
    if let Ok(parsed) = serde_json::from_str::<WsEnvelope>(payload)
        && parsed.channel.as_deref() == Some("ticker")
        && let Some(data) = parsed.data
        && let Some(last) = data.first().and_then(|d| d.last.as_ref())
        && let Ok(price) = last.parse::<Decimal>()
    {
        let mut guard = state.write().await;
        guard.push_price(price, window_size);
    }
}
