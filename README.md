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
# export KRAKEN_REST_BASE_URL=https://api.kraken.com
# export KRAKEN_STRATEGY=mean_reversion
# export KRAKEN_ANALYSIS_PAIRS='["UNI/USD","HYPE/USD","XMR/USD","SOL/USD","ETH/USD"]'
# export KRAKEN_RISK_AMOUNT_USD=10
# export KRAKEN_REWARD_TO_RISK=2
```

## Run

```bash
RUST_LOG=info cargo run --release -- BTC/USD
```

Only a pair symbol is required on the CLI. If omitted, Sentinel uses `BTC/USD`.

Supported analysis pairs (initial set):

- `UNI/USD`
- `HYPE/USD`
- `XMR/USD` (Monero)
- `SOL/USD`
- `ETH/USD`
- `BTC/USD`

Strategy selection is configurable through `KRAKEN_STRATEGY` with scaffolded options:

- `mean_reversion` (implemented)
- `breakout` (placeholder)
- `momentum` (placeholder)

## Behavior

1. Connects to Kraken WebSocket v2 (`wss://ws.kraken.com/v2`) and subscribes to ticker.
2. Updates rolling prices in shared state.
3. Every second computes mean/stddev over latest 20 points.
4. Signals:
   - `Z <= -2.0` => BUY
   - `Z >= 2.0` => SELL
5. Applies risk cap: max notional per trade is 5% of account balance.
6. Records each trade to sled (`trade:<timestamp_ms>`).
7. Computes and logs trading zones for each signal:
   - Stop Loss from configurable fixed risk budget (`KRAKEN_RISK_AMOUNT_USD`, default `10`).
   - Take Profit from configurable reward:risk ratio (`KRAKEN_REWARD_TO_RISK`, default `2`).

## Notes

- By default `dry_run=true`, so no live orders are sent.
- Live private REST order submission requires valid Kraken keys and base64 secret.
- Use `CTRL+C` for graceful shutdown. Both ingest and strategy loops are cancelled cleanly.

## Python Pivot Roadmap

This repository now includes a documented Python-first architecture for a 5m/15m/30m statistical mean-reversion stack focused on on-chain OHLCV, liquidity-state tracking, event-driven RAG, and local model inference.

See: [`docs/python_pivot_architecture.md`](docs/python_pivot_architecture.md).
