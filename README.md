# Sentinel Mean-Reversion Agent (Single Binary)

Sentinel is a production-oriented Rust trading agent built around:

- Async market-data ingestion with Tokio + tokio-tungstenite.
- Shared in-memory market state using `Arc<RwLock<MarketState>>`.
- Mean-reversion strategy (20-period rolling Z-score).
- Risk guardrails (5% exposure cap by default + kill switch on CTRL+C).
- Exact financial arithmetic with `rust_decimal`.
- Persistent trade journal via `sled`.
- Explainable structured logs using `tracing`.

## Setup

```bash
export KRAKEN_API_KEY="YourPublicKey"
export KRAKEN_API_SECRET="YourPrivateKey"
# Optional overrides:
# export KRAKEN_DRY_RUN=false
# export KRAKEN_DB_PATH=./sentinel_db
# export KRAKEN_WINDOW_SIZE=20
```

## Run

```bash
RUST_LOG=info cargo run --release -- BTC/USD
```

Only a pair symbol is required on the CLI. If omitted, Sentinel uses `BTC/USD`.

## Behavior

1. Connects to Kraken WebSocket v2 (`wss://ws.kraken.com/v2`) and subscribes to ticker.
2. Updates rolling prices in shared state.
3. Every second computes mean/stddev over latest 20 points.
4. Signals:
   - `Z <= -2.0` => BUY
   - `Z >= 2.0` => SELL
5. Applies risk cap: max notional per trade is 5% of account balance.
6. Records each trade to sled (`trade:<timestamp_ms>`).

## Notes

- By default `dry_run=true`, so no live orders are sent.
- Live private REST order submission requires valid Kraken keys and base64 secret.
- Use `CTRL+C` for graceful shutdown. Both ingest and strategy loops are cancelled cleanly.
