use anyhow::Result;
use config::{Config, Environment, File};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub kraken_api_key: Option<String>,
    pub kraken_api_secret: Option<String>,
    pub kraken_rest_base_url: String,
    pub ws_url: String,
    pub trade_pair: Option<String>,
    pub window_size: usize,
    pub z_buy_threshold: Decimal,
    pub z_sell_threshold: Decimal,
    pub max_exposure_pct: Decimal,
    pub dry_run: bool,
    pub db_path: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            kraken_api_key: None,
            kraken_api_secret: None,
            kraken_rest_base_url: "https://api.kraken.com".to_string(),
            ws_url: "wss://ws.kraken.com/v2".to_string(),
            trade_pair: None,
            window_size: 20,
            z_buy_threshold: Decimal::from_str("-2.0").expect("valid decimal"),
            z_sell_threshold: Decimal::from_str("2.0").expect("valid decimal"),
            max_exposure_pct: Decimal::from_str("0.05").expect("valid decimal"),
            dry_run: true,
            db_path: "./sentinel_db".to_string(),
        }
    }
}

pub fn load_settings() -> Result<Settings> {
    let defaults = Settings::default();
    let cfg = Config::builder()
        .set_default("kraken_rest_base_url", defaults.kraken_rest_base_url)?
        .set_default("ws_url", defaults.ws_url)?
        .set_default("window_size", defaults.window_size as i64)?
        .set_default("z_buy_threshold", defaults.z_buy_threshold.to_string())?
        .set_default("z_sell_threshold", defaults.z_sell_threshold.to_string())?
        .set_default("max_exposure_pct", defaults.max_exposure_pct.to_string())?
        .set_default("dry_run", defaults.dry_run)?
        .set_default("db_path", defaults.db_path)?
        .add_source(File::with_name("Sentinel").required(false))
        .add_source(Environment::with_prefix("KRAKEN").separator("_"))
        .build()?;

    Ok(cfg.try_deserialize()?)
}
