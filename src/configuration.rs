use anyhow::Result;
use config::{Config, Environment, File};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

use crate::strategy::StrategyKind;

pub const SUPPORTED_ANALYSIS_PAIRS: [&str; 6] = [
    "UNI/USD", "HYPE/USD", "XMR/USD", "SOL/USD", "ETH/USD", "BTC/USD",
];

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    #[serde(alias = "kraken_api_key")]
    pub api_key: Option<String>,
    #[serde(alias = "kraken_api_secret")]
    pub api_secret: Option<String>,
    #[serde(alias = "kraken_rest_base_url")]
    pub rest_base_url: String,
    pub ws_url: String,
    pub trade_pair: Option<String>,
    pub analysis_pairs: Vec<String>,
    pub strategy: String,
    pub window_size: usize,
    pub z_buy_threshold: Decimal,
    pub z_sell_threshold: Decimal,
    pub max_exposure_pct: Decimal,
    pub dry_run: bool,
    pub db_path: String,
    pub risk_amount_usd: Decimal,
    pub reward_to_risk: Decimal,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api_key: None,
            api_secret: None,
            rest_base_url: "https://api.kraken.com".to_string(),
            ws_url: "wss://ws.kraken.com/v2".to_string(),
            trade_pair: None,
            analysis_pairs: vec!["SOL/USD".to_string(), "ETH/USD".to_string()],
            strategy: "mean_reversion".to_string(),
            window_size: 20,
            z_buy_threshold: Decimal::from_str("-2.0").expect("valid decimal"),
            z_sell_threshold: Decimal::from_str("2.0").expect("valid decimal"),
            max_exposure_pct: Decimal::from_str("0.05").expect("valid decimal"),
            dry_run: true,
            db_path: "./sentinel_db".to_string(),
            risk_amount_usd: Decimal::from_str("10").expect("valid decimal"),
            reward_to_risk: Decimal::from_str("2").expect("valid decimal"),
        }
    }
}

pub fn load_settings() -> Result<Settings> {
    let defaults = Settings::default();
    let cfg = Config::builder()
        .set_default("rest_base_url", defaults.rest_base_url)?
        .set_default("ws_url", defaults.ws_url)?
        .set_default("analysis_pairs", defaults.analysis_pairs)?
        .set_default("strategy", defaults.strategy)?
        .set_default("window_size", defaults.window_size as i64)?
        .set_default("z_buy_threshold", defaults.z_buy_threshold.to_string())?
        .set_default("z_sell_threshold", defaults.z_sell_threshold.to_string())?
        .set_default("max_exposure_pct", defaults.max_exposure_pct.to_string())?
        .set_default("dry_run", defaults.dry_run)?
        .set_default("db_path", defaults.db_path)?
        .set_default("risk_amount_usd", defaults.risk_amount_usd.to_string())?
        .set_default("reward_to_risk", defaults.reward_to_risk.to_string())?
        .add_source(File::with_name("Sentinel").required(false))
        .add_source(Environment::with_prefix("KRAKEN").separator("_"))
        .build()?;

    Ok(cfg.try_deserialize()?)
}

pub fn parse_strategy(value: &str) -> Result<StrategyKind> {
    value.parse::<StrategyKind>().map_err(anyhow::Error::msg)
}

pub fn normalize_pair_symbol(pair: &str) -> String {
    pair.trim().to_ascii_uppercase().replace('-', "/")
}

pub fn is_supported_analysis_pair(pair: &str) -> bool {
    let normalized = normalize_pair_symbol(pair);
    SUPPORTED_ANALYSIS_PAIRS.contains(&normalized.as_str())
}
