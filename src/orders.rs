use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, Mac};
use reqwest::Client;
use rust_decimal::Decimal;
use sha2::{Digest, Sha256, Sha512};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::configuration::Settings;

#[derive(Clone)]
pub struct OrderExecutor {
    client: Client,
    settings: Settings,
}

impl OrderExecutor {
    pub fn new(settings: Settings) -> Self {
        Self {
            client: Client::new(),
            settings,
        }
    }

    pub async fn get_account_balance(&self) -> Result<Decimal> {
        if self.settings.dry_run {
            return Ok(Decimal::from(10_000));
        }
        // Placeholder for Kraken private REST call.
        Ok(Decimal::from(10_000))
    }

    pub async fn place_market_order(&self, pair: &str, side: &str, volume: Decimal) -> Result<()> {
        if self.settings.dry_run {
            tracing::info!(pair, side, volume=%volume, "Dry-run mode: not sending REST order");
            return Ok(());
        }

        let key = self
            .settings
            .api_key
            .clone()
            .context("KRAKEN_API_KEY missing")?;
        let secret = self
            .settings
            .api_secret
            .clone()
            .context("KRAKEN_API_SECRET missing")?;

        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let post_data =
            format!("nonce={nonce}&ordertype=market&type={side}&pair={pair}&volume={volume}");
        let path = "/0/private/AddOrder";
        let signature = kraken_signature(path, nonce, &post_data, &secret)?;
        let url = format!("{}{}", self.settings.rest_base_url, path);

        self.client
            .post(url)
            .header("API-Key", key)
            .header("API-Sign", signature)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(post_data)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn cancel_all_open_orders(&self) -> Result<()> {
        if self.settings.dry_run {
            tracing::info!("Dry-run mode: cancel_all_open_orders skipped");
            return Ok(());
        }
        // Placeholder for Kraken CancelAll endpoint integration.
        Ok(())
    }
}

fn kraken_signature(path: &str, nonce: u128, post_data: &str, secret_b64: &str) -> Result<String> {
    let mut sha256 = Sha256::new();
    sha256.update(format!("{nonce}{post_data}"));
    let hash = sha256.finalize();

    let mut data = Vec::with_capacity(path.len() + hash.len());
    data.extend_from_slice(path.as_bytes());
    data.extend_from_slice(&hash);

    let secret = general_purpose::STANDARD.decode(secret_b64)?;
    let mut hmac = Hmac::<Sha512>::new_from_slice(&secret)?;
    hmac.update(&data);
    let sig = hmac.finalize().into_bytes();
    Ok(general_purpose::STANDARD.encode(sig))
}
