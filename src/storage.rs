use anyhow::Result;
use serde::{Deserialize, Serialize};
use sled::Db;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub timestamp_ms: u128,
    pub pair: String,
    pub side: String,
    pub price: String,
    pub z_score: String,
    pub reason: String,
}

#[derive(Clone)]
pub struct Storage {
    db: Db,
}

impl Storage {
    pub fn open(path: &str) -> Result<Self> {
        Ok(Self {
            db: sled::open(path)?,
        })
    }

    pub fn save_trade(&self, record: &TradeRecord) -> Result<()> {
        let key = format!("trade:{}", record.timestamp_ms);
        let val = serde_json::to_vec(record)?;
        self.db.insert(key.as_bytes(), val)?;
        self.db.flush()?;
        Ok(())
    }
}
