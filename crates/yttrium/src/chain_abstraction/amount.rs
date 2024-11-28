use alloy::primitives::{
    utils::{ParseUnits, Unit},
    U256,
};

uniffi::custom_type!(Unit, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

#[derive(Debug)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct Amount {
    pub symbol: String,        // USDC, USD
    pub amount: U256,          // e.g. 40000, 4
    pub unit: Unit,            // 6, 2
    pub formatted: String,     // e.g. 0.04 USDC, $0.04
    pub formatted_alt: String, // e.g. $0.04
}

impl Amount {
    pub fn new(symbol: String, amount: U256, unit: Unit) -> Self {
        let formatted = ParseUnits::U256(amount).format_units(unit);
        let formatted_symbol = format!("{formatted} {symbol}");
        let formatted_alt = {
            let unit_offset = U256::from(10).pow(U256::from(unit.get() - 2));
            let decimals = amount % unit_offset;
            let mut amount = amount / unit_offset;
            if decimals > unit_offset / U256::from(2) {
                // round up
                amount += U256::from(1);
            }
            if amount.is_zero() && !decimals.is_zero() {
                "<$0.01".to_owned()
            } else {
                let formatted = ParseUnits::U256(amount)
                    .format_units(Unit::new(2).unwrap());
                format!("${formatted}")
            }
        };
        Self {
            symbol,
            amount,
            unit,
            formatted: formatted_symbol,
            formatted_alt,
        }
    }

    pub fn zero() -> Self {
        Self::new("UNK".to_string(), U256::from(0), Unit::new(0).unwrap())
    }
}

impl Default for Amount {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_whole() {
        let amount = Amount::new(
            "USDC".to_string(),
            U256::from(4000000),
            Unit::new(6).unwrap(),
        );
        assert_eq!(amount.formatted, "4.000000 USDC");
        assert_eq!(amount.formatted_alt, "$4.00");
    }

    #[test]
    fn test_amount_zero() {
        let amount = Amount::new(
            "USDC".to_string(),
            U256::from(0),
            Unit::new(6).unwrap(),
        );
        assert_eq!(amount.formatted, "0.000000 USDC");
        assert_eq!(amount.formatted_alt, "$0.00");
    }

    #[test]
    fn test_amount_cents() {
        let amount = Amount::new(
            "USDC".to_string(),
            U256::from(40000),
            Unit::new(6).unwrap(),
        );
        assert_eq!(amount.formatted, "0.040000 USDC");
        assert_eq!(amount.formatted_alt, "$0.04");
    }

    #[test]
    fn test_amount_less_than_1_but_round_up() {
        let amount = Amount::new(
            "USDC".to_string(),
            U256::from(9000),
            Unit::new(6).unwrap(),
        );
        assert_eq!(amount.formatted, "0.009000 USDC");
        assert_eq!(amount.formatted_alt, "$0.01");
    }

    #[test]
    fn test_amount_less_than_1() {
        let amount = Amount::new(
            "USDC".to_string(),
            U256::from(4000),
            Unit::new(6).unwrap(),
        );
        assert_eq!(amount.formatted, "0.004000 USDC");
        assert_eq!(amount.formatted_alt, "<$0.01");

        let amount = Amount::new(
            "USDC".to_string(),
            U256::from(100),
            Unit::new(6).unwrap(),
        );
        assert_eq!(amount.formatted, "0.000100 USDC");
        assert_eq!(amount.formatted_alt, "<$0.01");

        let amount = Amount::new(
            "USDC".to_string(),
            U256::from(1),
            Unit::new(6).unwrap(),
        );
        assert_eq!(amount.formatted, "0.000001 USDC");
        assert_eq!(amount.formatted_alt, "<$0.01");
    }
}
