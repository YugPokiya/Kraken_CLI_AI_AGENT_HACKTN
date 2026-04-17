mod configuration;
mod kraken_ws;
mod orders;
mod state;
mod storage;
mod strategy;

use anyhow::Result;
use rust_decimal::Decimal;
use std::env;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::EnvFilter;

use configuration::load_settings;
use orders::OrderExecutor;
use state::{MarketState, SharedMarketState};
use storage::{Storage, TradeRecord};
use strategy::{TradeSignal, calculate_z_score, compute_trading_zones, signal_from_z_score};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let mut settings = load_settings()?;
    let pair = env::args()
        .nth(1)
        .or(settings.trade_pair.clone())
        .unwrap_or_else(|| "BTC/USD".to_string());
    let pair = configuration::normalize_pair_symbol(&pair);
    settings.trade_pair = Some(pair.clone());

    if !configuration::is_supported_analysis_pair(&pair) {
        anyhow::bail!(
            "Unsupported pair '{}'. Supported analysis pairs: {:?}",
            pair,
            configuration::SUPPORTED_ANALYSIS_PAIRS
        );
    }

    let strategy = configuration::parse_strategy(&settings.strategy)?;
    tracing::info!(
        ?strategy,
        analysis_pairs=?settings.analysis_pairs,
        risk_amount_usd=%settings.risk_amount_usd,
        reward_to_risk=%settings.reward_to_risk,
        "Trading configuration loaded"
    );

    let market_state: SharedMarketState = Arc::new(RwLock::new(MarketState::new(
        pair.clone(),
        settings.window_size,
    )));
    let storage = Storage::open(&settings.db_path)?;
    let order_executor = OrderExecutor::new(settings.clone());

    let shutdown = CancellationToken::new();
    let ws_shutdown = shutdown.child_token();
    let strat_shutdown = shutdown.child_token();

    let ws_state = Arc::clone(&market_state);
    let ws_settings = settings.clone();
    let ws_handle = tokio::spawn(async move {
        if let Err(e) = kraken_ws::run_market_data_ingest(ws_state, ws_settings, ws_shutdown).await
        {
            tracing::error!(error=%e, "WebSocket ingest failed");
        }
    });

    let strat_state = Arc::clone(&market_state);
    let strat_settings = settings.clone();
    let strat_executor = order_executor.clone();
    let strat_handle = tokio::spawn(async move {
        if let Err(e) = run_strategy_loop(
            strat_state,
            storage,
            strat_executor,
            strat_settings,
            strat_shutdown,
        )
        .await
        {
            tracing::error!(error=%e, "Strategy loop failed");
        }
    });

    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutdown signal received, cancelling tasks");
    shutdown.cancel();
    order_executor.cancel_all_open_orders().await?;

    let _ = tokio::join!(ws_handle, strat_handle);
    Ok(())
}

async fn run_strategy_loop(
    state: SharedMarketState,
    storage: Storage,
    executor: OrderExecutor,
    settings: configuration::Settings,
    shutdown: CancellationToken,
) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("Strategy loop received shutdown signal");
                break;
            }
            _ = interval.tick() => {
                let (pair, prices) = {
                    let guard = state.read().await;
                    (guard.pair.clone(), guard.prices.iter().copied().collect::<Vec<Decimal>>())
                };

                if let Some(ctx) = calculate_z_score(&prices, settings.window_size) {
                    let signal = signal_from_z_score(&ctx, settings.z_buy_threshold, settings.z_sell_threshold);
                    if signal != TradeSignal::Hold {
                        let balance = executor.get_account_balance().await?;
                        let notional = balance * settings.max_exposure_pct;
                        let volume = if ctx.current_price > Decimal::ZERO {
                            notional / ctx.current_price
                        } else {
                            Decimal::ZERO
                        };

                        if volume <= Decimal::ZERO {
                            tracing::warn!("Signal ignored due to zero volume after risk checks");
                            continue;
                        }

                        if let Some(zones) = compute_trading_zones(
                            signal,
                            ctx.current_price,
                            settings.risk_amount_usd,
                            volume,
                            settings.reward_to_risk,
                        ) {
                            tracing::info!(
                                pair,
                                signal=?signal,
                                stop_loss=%zones.stop_loss,
                                take_profit=%zones.take_profit,
                                "Computed trading zones"
                            );
                        }

                        let side = if signal == TradeSignal::Buy { "buy" } else { "sell" };
                        tracing::info!(
                            pair,
                            side,
                            price=%ctx.current_price,
                            z_score=%ctx.z_score,
                            mean=%ctx.mean,
                            std_dev=%ctx.std_dev,
                            exposure_pct=%settings.max_exposure_pct,
                            "Executing mean-reversion trade"
                        );

                        executor.place_market_order(&pair, side, volume).await?;

                        let record = TradeRecord {
                            timestamp_ms: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
                            pair: pair.clone(),
                            side: side.to_string(),
                            price: ctx.current_price.to_string(),
                            z_score: ctx.z_score.to_string(),
                            reason: format!("z={} mean={} std={}", ctx.z_score, ctx.mean, ctx.std_dev),
                        };
                        storage.save_trade(&record)?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
