use alloy::primitives::{utils::Unit, U256};

struct Fee {
    fungible_amount: U256,
    fungible_decimals: u8,
    fungible_price: U256,
    fungible_price_decimals: u8,
}

// A helper to get the total value of a list of asset amounts in a user's local
// currency without incurring floating point errors Add many amounts with add(),
// providing:
// - the asset amount and unit e.g. 1 ETH with 18 decimals, inputted as
//   1_000_000_000_000_000_000, Unit(18)
// - and the price of the asset and its unit e.g. 4000 USD with 2 decimals,
//   inputted as 400000, Unit(2)
// When amounts have been added with add(), call compute() to get the total
// amount and its unit. E.g. (amount/10^unit)=4000 USD The local currency
// exchange rate must the same for all assets (e.g. USD) This works by finding
// the maximum of the number of decimals for the asset amount and price, and
// adjusting all entries up to that level of decimals
pub struct LocalAmountAcc {
    fees: Vec<Fee>,
}

impl LocalAmountAcc {
    pub fn new() -> Self {
        Self { fees: Vec::new() }
    }

    pub fn add(
        &mut self,
        fungible_amount: U256,
        fungible_decimals: Unit,
        fungible_price: U256,
        fungible_price_decimals: Unit,
    ) {
        self.fees.push(Fee {
            fungible_amount,
            fungible_decimals: fungible_decimals.get(),
            fungible_price,
            fungible_price_decimals: fungible_price_decimals.get(),
        });
    }

    pub fn compute(&self) -> (U256, Unit) {
        let max_fungible_decimals = self
            .fees
            .iter()
            .map(|fee| fee.fungible_decimals)
            .max()
            .unwrap_or(0);
        let max_fungible_price_decimals = self
            .fees
            .iter()
            .map(|fee| fee.fungible_price_decimals)
            .max()
            .unwrap_or(0);
        let decimals = max_fungible_decimals + max_fungible_price_decimals;
        let mut total_local_fee_acc = U256::ZERO;
        for fee in &self.fees {
            let adjusted_fungible_amount = fee.fungible_amount
                * U256::from(10).pow(U256::from(
                    max_fungible_decimals - fee.fungible_decimals,
                ));
            let adjusted_fungible_price = fee.fungible_price
                * U256::from(10).pow(U256::from(
                    max_fungible_price_decimals - fee.fungible_price_decimals,
                ));
            total_local_fee_acc +=
                adjusted_fungible_amount * adjusted_fungible_price;
        }
        (total_local_fee_acc, Unit::new(decimals).unwrap())
    }
}

impl Default for LocalAmountAcc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::chain_abstraction::{
            amount::{from_float, to_float},
            test_helpers::floats_close,
        },
    };

    #[test]
    fn zero_fee() {
        let (amount, decimals) = LocalAmountAcc::new().compute();
        assert!(amount.is_zero());
        assert_eq!(decimals.get(), 0);
    }

    #[test]
    fn one_usdc_simple_price() {
        let mut acc = LocalAmountAcc::new();
        acc.add(
            U256::from(1000000),   // 1 USDC
            Unit::new(6).unwrap(), // 6 decimals in USDC
            U256::from(1),         // 1 = 1
            Unit::new(0).unwrap(), // no decimals
        );
        let (amount, decimals) = acc.compute();
        assert_eq!(amount, U256::from(1000000));
        assert_eq!(decimals.get(), 6);
    }

    #[test]
    fn one_usdc_detailed_price() {
        let mut acc = LocalAmountAcc::new();
        acc.add(
            U256::from(1000000),   // 1 USDC
            Unit::new(6).unwrap(), // 6 decimals in USDC
            U256::from(100),       // 1 = 1
            Unit::new(2).unwrap(), // no decimals
        );
        let (amount, decimals) = acc.compute();
        assert_eq!(amount, U256::from(100000000));
        assert_eq!(decimals.get(), 8);
    }

    #[test]
    fn two_usdc_detailed_price() {
        let mut acc = LocalAmountAcc::new();
        acc.add(
            U256::from(2000000),   // 2 USDC
            Unit::new(6).unwrap(), // 6 decimals in USDC
            U256::from(100),       // 1 USDC = $1.00
            Unit::new(2).unwrap(), // 2 decimals
        );
        let (amount, decimals) = acc.compute();
        assert_eq!(amount, U256::from(200000000));
        assert_eq!(decimals.get(), 8);
    }

    #[test]
    fn detailed_price() {
        let mut acc = LocalAmountAcc::new();
        let (fungible_amount, fungible_decimals) = from_float(1., 0);
        let (fungible_price, fungible_price_decimals) = from_float(4000., 0);
        acc.add(
            fungible_amount,
            fungible_decimals,
            fungible_price,
            fungible_price_decimals,
        );
        let (amount, decimals) = acc.compute();
        let amount = to_float(amount, decimals);
        assert_eq!(amount, 4000.);
    }

    #[test]
    fn detailed_price2() {
        let mut acc = LocalAmountAcc::new();
        let (fungible_amount, fungible_decimals) = from_float(0.01, 9);
        let (fungible_price, fungible_price_decimals) = from_float(4000., 0);
        acc.add(
            fungible_amount,
            fungible_decimals,
            fungible_price,
            fungible_price_decimals,
        );
        let (amount, decimals) = acc.compute();
        let amount = to_float(amount, decimals);
        assert_eq!(amount, 40.);
    }

    #[test]
    fn usdc_eth_detailed() {
        let mut acc = LocalAmountAcc::new();

        // ETH
        let (fungible_amount, fungible_decimals) = from_float(0.01, 18);
        let (fungible_price, fungible_price_decimals) = from_float(4000., 6);
        acc.add(
            fungible_amount,
            fungible_decimals,
            fungible_price,
            fungible_price_decimals,
        );

        // USDC
        let (fungible_amount, fungible_decimals) = from_float(2., 6);
        let (fungible_price, fungible_price_decimals) = from_float(1., 0);
        acc.add(
            fungible_amount,
            fungible_decimals,
            fungible_price,
            fungible_price_decimals,
        );

        let (amount, decimals) = acc.compute();
        let amount = to_float(amount, decimals);
        assert_eq!(amount, 42.);
    }

    #[test]
    fn usdc_eth_detailed2() {
        let mut acc = LocalAmountAcc::new();

        // ETH
        let (fungible_amount, fungible_decimals) = from_float(0.01, 18);
        let (fungible_price, fungible_price_decimals) = from_float(4000., 6);
        acc.add(
            fungible_amount,
            fungible_decimals,
            fungible_price,
            fungible_price_decimals,
        );

        // USDC
        let (fungible_amount, fungible_decimals) = from_float(2., 6);
        let (fungible_price, fungible_price_decimals) = from_float(1., 10);
        acc.add(
            fungible_amount,
            fungible_decimals,
            fungible_price,
            fungible_price_decimals,
        );

        let (amount, decimals) = acc.compute();
        let amount = to_float(amount, decimals);
        assert!(floats_close(amount, 42., 0.0000001));
    }

    #[test]
    fn usdc_eth_detailed3() {
        let mut acc = LocalAmountAcc::new();

        // ETH
        let (fungible_amount, fungible_decimals) = from_float(0.01, 18);
        let (fungible_price, fungible_price_decimals) = from_float(4000., 6);
        acc.add(
            fungible_amount,
            fungible_decimals,
            fungible_price,
            fungible_price_decimals,
        );
        let (amount, decimals) = acc.compute();
        let amount = to_float(amount, decimals);
        println!("amount1: {amount}");

        // Another amount of ETH
        let (fungible_amount, fungible_decimals) = from_float(0.02, 18);
        let (fungible_price, fungible_price_decimals) = from_float(4000., 6);
        acc.add(
            fungible_amount,
            fungible_decimals,
            fungible_price,
            fungible_price_decimals,
        );
        let (amount, decimals) = acc.compute();
        let amount = to_float(amount, decimals);
        println!("amount2: {amount}");

        // USDC
        let (fungible_amount, fungible_decimals) = from_float(2., 6);
        let (fungible_price, fungible_price_decimals) = from_float(1., 10);
        acc.add(
            fungible_amount,
            fungible_decimals,
            fungible_price,
            fungible_price_decimals,
        );
        let (amount, decimals) = acc.compute();
        let amount = to_float(amount, decimals);
        println!("amount3: {amount}");

        // DOGE
        let (fungible_amount, fungible_decimals) = from_float(3., 6);
        let (fungible_price, fungible_price_decimals) = from_float(0.00001, 10);
        acc.add(
            fungible_amount,
            fungible_decimals,
            fungible_price,
            fungible_price_decimals,
        );
        let (amount, decimals) = acc.compute();
        let amount = to_float(amount, decimals);
        println!("amount4: {amount}");

        let (amount, decimals) = acc.compute();
        let amount = to_float(amount, decimals);
        println!("amount: {amount}");
        assert!(floats_close(amount, 122.00003, 0.000000001));
    }

    #[test]
    fn test_from_float() {
        assert_eq!(
            from_float(1., Unit::ETHER.get()),
            (U256::from(1_000_000_000_000_000_000_u128), Unit::ETHER)
        );
        assert_eq!(
            from_float(1., 6),
            (U256::from(1_000_000_u128), Unit::new(6).unwrap())
        );
    }

    #[test]
    fn test_to_float() {
        assert_eq!(
            to_float(U256::from(1_000_000_000_000_000_000_u128), Unit::ETHER),
            1.
        );
        assert_eq!(
            to_float(U256::from(1_000_000_u128), Unit::new(6).unwrap()),
            1.
        );
        assert_eq!(
            to_float(U256::from(2_000_000_000_000_000_000_u128), Unit::ETHER),
            2.
        );
        assert_eq!(
            to_float(U256::from(2_000_000_u128), Unit::new(6).unwrap()),
            2.
        );
        assert_eq!(
            to_float(U256::from(2_000_000_000_000_000_u128), Unit::ETHER),
            0.002
        );
        assert_eq!(
            to_float(U256::from(2_000_u128), Unit::new(6).unwrap()),
            0.002
        );
    }
}
