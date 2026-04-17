use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct SignalContext {
    pub current_price: Decimal,
    pub mean: Decimal,
    pub std_dev: Decimal,
    pub z_score: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeSignal {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyKind {
    MeanReversion,
    Breakout,
    Momentum,
}

impl FromStr for StrategyKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "mean_reversion" | "mean-reversion" | "meanreversion" => Ok(Self::MeanReversion),
            "breakout" => Ok(Self::Breakout),
            "momentum" => Ok(Self::Momentum),
            _ => Err(format!("unsupported strategy: {value}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TradingZones {
    pub stop_loss: Decimal,
    pub take_profit: Decimal,
}

pub fn compute_trading_zones(
    signal: TradeSignal,
    entry_price: Decimal,
    risk_amount_usd: Decimal,
    position_size: Decimal,
    reward_to_risk: Decimal,
) -> Option<TradingZones> {
    if entry_price <= Decimal::ZERO
        || risk_amount_usd <= Decimal::ZERO
        || position_size <= Decimal::ZERO
        || reward_to_risk <= Decimal::ZERO
        || signal == TradeSignal::Hold
    {
        return None;
    }

    let risk_per_unit = risk_amount_usd / position_size;
    let reward_per_unit = risk_per_unit * reward_to_risk;

    let (stop_loss, take_profit) = match signal {
        TradeSignal::Buy => (entry_price - risk_per_unit, entry_price + reward_per_unit),
        TradeSignal::Sell => (entry_price + risk_per_unit, entry_price - reward_per_unit),
        TradeSignal::Hold => return None,
    };

    Some(TradingZones {
        stop_loss,
        take_profit,
    })
}

pub fn calculate_z_score(window: &[Decimal], min_window: usize) -> Option<SignalContext> {
    if window.len() < min_window || window.len() < 2 {
        return None;
    }

    let n = Decimal::from(window.len() as u64);
    let n_minus_1 = Decimal::from((window.len() - 1) as u64);
    let sum: Decimal = window.iter().copied().sum();
    let mean = sum / n;

    let variance = window
        .iter()
        .map(|v| {
            let diff = *v - mean;
            diff * diff
        })
        .sum::<Decimal>()
        / n_minus_1;

    let variance_f = variance.to_f64()?;
    let std_dev_f = variance_f.sqrt();
    let std_dev = Decimal::from_f64_retain(std_dev_f)?;

    if std_dev == dec!(0) {
        return None;
    }

    let current_price = *window.last()?;
    let z_score = (current_price - mean) / std_dev;

    Some(SignalContext {
        current_price,
        mean,
        std_dev,
        z_score,
    })
}

pub fn signal_from_z_score(
    ctx: &SignalContext,
    buy_threshold: Decimal,
    sell_threshold: Decimal,
) -> TradeSignal {
    if ctx.z_score <= buy_threshold {
        TradeSignal::Buy
    } else if ctx.z_score >= sell_threshold {
        TradeSignal::Sell
    } else {
        TradeSignal::Hold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_buy_signal_on_large_negative_deviation() {
        let mut prices = vec![dec!(100); 19];
        prices.push(dec!(90));
        let ctx = calculate_z_score(&prices, 20).expect("zscore");
        assert_eq!(
            signal_from_z_score(&ctx, dec!(-2), dec!(2)),
            TradeSignal::Buy
        );
    }

    #[test]
    fn returns_none_for_small_window() {
        let prices = vec![dec!(100); 5];
        assert!(calculate_z_score(&prices, 20).is_none());
    }

    #[test]
    fn parses_supported_strategy_kinds() {
        assert_eq!(
            "mean_reversion".parse::<StrategyKind>().unwrap(),
            StrategyKind::MeanReversion
        );
        assert_eq!(
            "breakout".parse::<StrategyKind>().unwrap(),
            StrategyKind::Breakout
        );
        assert_eq!(
            "momentum".parse::<StrategyKind>().unwrap(),
            StrategyKind::Momentum
        );
    }

    #[test]
    fn computes_trading_zones_for_buy_signal() {
        let zones =
            compute_trading_zones(TradeSignal::Buy, dec!(100), dec!(10), dec!(2), dec!(2)).unwrap();
        assert_eq!(zones.stop_loss, dec!(95));
        assert_eq!(zones.take_profit, dec!(110));
    }
}
