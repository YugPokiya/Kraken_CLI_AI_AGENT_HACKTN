# Python Statistical Pivot Architecture (5m / 15m / 30m)

## Why this pivot works

Moving from tick/mempool competition to 5, 15, and 30-minute decision windows shifts the system from latency-first engineering to signal-quality-first engineering. This favors a local, Python-native quantitative stack focused on:

- deterministic OHLCV aggregation,
- statistical anomaly detection in AMM liquidity,
- explainable event-driven retrieval and risk narratives.

## 1) Core statistical model

### 1.1 Time-series aggregation (OHLCV)

- Build canonical 5m, 15m, and 30m candles from finalized on-chain trades.
- Candle close is the checkpoint for feature calculation and model inference.
- Compute rolling features on each interval:
  - returns / log-returns,
  - z-scores,
  - realized volatility,
  - volume delta,
  - RSI / Bollinger / Fisher Transform.

### 1.2 Liquidity state tracking

At each candle close, snapshot pool state for target AMMs (e.g., Uniswap V3):

- token reserves / virtual liquidity,
- price tick ranges and depth proxies,
- imbalance metrics between pool sides,
- slippage estimates for standardized trade sizes.

These become additional explanatory features for mean-reversion and regime detection.

### 1.3 Event-driven RAG

Trigger retrieval and narrative generation only on significant events, such as:

- 15m candle volume z-score > threshold,
- liquidity depth collapse over one interval,
- repeated whale-sized swap clusters within 30m.

This avoids constant LLM invocation and keeps local inference efficient.

## 2) Python-first local stack

### 2.1 Data ingestion + transformation

- **Python** as the primary runtime.
- **Web3.py** for finalized blocks, logs, and pool state reads.
- **Polars** (preferred) for fast, vectorized rolling analytics.

Implementation notes:

- process block ranges in idempotent batches,
- persist last processed block per data source,
- reconcile reorg-sensitive intervals conservatively.

### 2.2 Storage and memory

- **PostgreSQL (Docker)** for structured market and feature tables.
- **ChromaDB (Docker)** for embeddings of wallet behavior, anomaly context, and prior incident summaries.

Suggested PostgreSQL entities:

- `candles_5m`, `candles_15m`, `candles_30m`,
- `pool_snapshots`,
- `features_<interval>`,
- `signals`,
- `model_predictions`,
- `rag_events`.

### 2.3 Modeling + AI

- **PyTorch** for short-horizon sequence models (e.g., GRU) over 15m/30m features.
- **pandas-ta or TA-Lib** for feature engineering.
- **Ollama** for local LLM serving and low-cost RAG generation.

### 2.4 API layer

- **FastAPI + Uvicorn** exposes:
  - latest candles/features,
  - model forecasts and confidence,
  - triggered anomaly events,
  - generated risk narratives.

## 3) Recommended project layout

```text
python-agent/
  app/
    api/
      main.py
      routes/
    ingestion/
      web3_client.py
      block_sync.py
      ohlcv_builder.py
    features/
      indicators.py
      liquidity_metrics.py
    models/
      dataset.py
      gru_model.py
      train.py
      infer.py
    rag/
      embeddings.py
      retriever.py
      generator.py
      triggers.py
    storage/
      postgres.py
      chroma.py
      schema.sql
    core/
      settings.py
      logging.py
  docker-compose.yml
  pyproject.toml
  README.md
```

## 4) Dataflow at runtime

1. Sync finalized blocks and relevant AMM events.
2. Build/update 5m/15m/30m candles.
3. Snapshot pool liquidity state at candle close.
4. Compute feature vectors + regime/anomaly scores.
5. Run model inference for next-interval directional/liquidity forecast.
6. If trigger conditions are met, run retrieval + local LLM narrative.
7. Persist outputs and expose through FastAPI.

## 5) Migration path from the current Rust binary

1. Keep the Rust agent stable for exchange-centric execution.
2. Stand up a parallel Python service for on-chain statistical research.
3. Validate feature parity and signal quality in paper mode.
4. Promote Python forecasts to production-facing API endpoints.
5. Optionally retain Rust only for execution/risk microservice boundaries.

## 6) Immediate next tasks

- [ ] Bootstrap `python-agent/` with FastAPI, Polars, Web3.py, and PyTorch.
- [ ] Define PostgreSQL schema and migration scripts.
- [ ] Implement 5m/15m/30m candle builder and feature pipeline.
- [ ] Add first anomaly trigger (volume z-score + liquidity drop).
- [ ] Integrate Ollama + ChromaDB for event-driven narratives.
- [ ] Add backtesting notebook for mean-reversion quality checks.
