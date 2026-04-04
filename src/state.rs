use rust_decimal::Decimal;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct MarketState {
    pub pair: String,
    pub latest_price: Option<Decimal>,
    pub prices: VecDeque<Decimal>,
}

impl MarketState {
    pub fn new(pair: String, window_size: usize) -> Self {
        Self {
            pair,
            latest_price: None,
            prices: VecDeque::with_capacity(window_size),
        }
    }

    pub fn push_price(&mut self, price: Decimal, window_size: usize) {
        self.latest_price = Some(price);
        self.prices.push_back(price);
        while self.prices.len() > window_size {
            self.prices.pop_front();
        }
    }
}

pub type SharedMarketState = Arc<RwLock<MarketState>>;
