use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;

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
}
