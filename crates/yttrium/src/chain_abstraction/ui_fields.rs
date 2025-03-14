use {
    super::{
        amount::Amount,
        api::{
            prepare::PrepareResponseAvailable, FeeEstimatedTransaction,
            Transaction,
        },
    },
    crate::chain_abstraction::{
        amount::from_float,
        api::fungible_price::{FungiblePriceItem, NATIVE_TOKEN_ADDRESS},
        local_fee_acc::LocalAmountAcc,
    },
    alloy::primitives::{PrimitiveSignature, B256, U256},
    alloy_provider::utils::Eip1559Estimation,
    serde::{Deserialize, Serialize},
    tracing::warn,
};
#[cfg(feature = "solana")]
use {
    crate::chain_abstraction::{api::prepare::SolanaTransaction, solana},
    alloy::primitives::Bytes,
};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct UiFields {
    pub route_response: PrepareResponseAvailable,
    pub route: Vec<Route>,
    pub local_route_total: Amount,
    pub bridge: Vec<TransactionFee>,
    pub local_bridge_total: Amount,
    pub initial: TxnDetails,
    pub local_total: Amount,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase", tag = "namespace")]
pub enum Route {
    #[cfg(feature = "eip155")]
    Eip155(Vec<TxnDetails>),
    #[cfg(feature = "solana")]
    Solana(Vec<SolanaTxnDetails>),
}

impl Route {
    #[cfg(feature = "eip155")]
    pub fn into_eip155(self) -> Option<Vec<TxnDetails>> {
        match self {
            Self::Eip155(route) => Some(route),
            #[cfg(feature = "solana")]
            Self::Solana(_) => None,
        }
    }

    #[cfg(feature = "eip155")]
    pub fn as_eip155(&self) -> Option<&Vec<TxnDetails>> {
        match self {
            Self::Eip155(route) => Some(route),
            #[cfg(feature = "solana")]
            Self::Solana(_) => None,
        }
    }

    #[cfg(feature = "solana")]
    pub fn into_solana(self) -> Option<Vec<SolanaTxnDetails>> {
        match self {
            Self::Solana(route) => Some(route),
            Self::Eip155(_) => None,
        }
    }

    #[cfg(feature = "solana")]
    pub fn as_solana(&self) -> Option<&Vec<SolanaTxnDetails>> {
        match self {
            Self::Solana(route) => Some(route),
            Self::Eip155(_) => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct TxnDetails {
    pub transaction: FeeEstimatedTransaction,
    pub transaction_hash_to_sign: B256,
    pub fee: TransactionFee,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct TransactionFee {
    pub fee: Amount,
    pub local_fee: Amount,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "solana")]
pub struct SolanaTxnDetails {
    pub transaction: SolanaTransaction,
    pub transaction_hash_to_sign: Bytes,
    // pub fee: TransactionFee,
}

pub enum EstimatedRouteTransaction {
    #[cfg(feature = "eip155")]
    Eip155(Vec<(Transaction, Eip1559Estimation, U256)>),
    #[cfg(feature = "solana")]
    Solana(Vec<(SolanaTransaction,)>),
}

pub fn ui_fields(
    route_response: PrepareResponseAvailable,
    estimated_transactions: Vec<EstimatedRouteTransaction>,
    estimated_initial_transaction: (Transaction, Eip1559Estimation, U256),
    fungibles: Vec<FungiblePriceItem>,
) -> UiFields {
    let mut total_local_fee = LocalAmountAcc::new();
    let mut local_route_total_acc = LocalAmountAcc::new();
    let mut local_bridge_total_acc = LocalAmountAcc::new();

    fn compute_amounts(
        fee: U256,
        total_local_fee: &mut Vec<&mut LocalAmountAcc>,
        fungible: &FungiblePriceItem,
    ) -> TransactionFee {
        // `fungible.price` is a float; with obviously floating-point so should have great precision
        // Set this value to a value that is high enough to capture the desired price movement
        // Setting it too high may overflow the 77 decimal places (Unit::MAX) of the U256
        // Some tokens such as ETH only need 2 decimal places because their value is very high (>1000) and price moves are large
        // Some tokens may be worth e.g. 0.000001 USD per token, so we need to capture more decimal places to even see price movement
        const FUNGIBLE_PRICE_PRECISION: u8 = 8;

        let (fungible_price, fungible_price_decimals) =
            from_float(fungible.price, FUNGIBLE_PRICE_PRECISION);

        for total_local_fee in total_local_fee {
            total_local_fee.add(
                fee,
                fungible.decimals,
                fungible_price,
                fungible_price_decimals,
            );
        }

        let mut local_fee = LocalAmountAcc::new();
        local_fee.add(
            fee,
            fungible.decimals,
            fungible_price,
            fungible_price_decimals,
        );
        let (local_fee, local_fee_unit) = local_fee.compute();

        TransactionFee {
            fee: Amount::new(fungible.symbol.clone(), fee, fungible.decimals),
            local_fee: Amount::new("USD".to_owned(), local_fee, local_fee_unit),
        }
    }

    let mut routes = Vec::new();

    for estimated_route_transactions in estimated_transactions {
        match estimated_route_transactions {
            #[cfg(feature = "eip155")]
            EstimatedRouteTransaction::Eip155(estimated_transactions) => {
                let mut route =
                    Vec::with_capacity(estimated_transactions.len());
                for item in estimated_transactions {
                    let fee = compute_amounts(
                        item.2,
                        &mut vec![
                            &mut total_local_fee,
                            &mut local_route_total_acc,
                        ],
                        fungibles
                            .iter()
                            .find(|f| {
                                f.address
                                    == format!(
                                        "{}:{}",
                                        item.0.chain_id,
                                        NATIVE_TOKEN_ADDRESS.to_checksum(None)
                                    )
                            })
                            .unwrap(),
                    );
                    let transaction =
                        FeeEstimatedTransaction::from_transaction_and_estimate(
                            item.0, item.1,
                        );
                    route.push(TxnDetails {
                        transaction_hash_to_sign: transaction
                            .clone()
                            .into_signing_hash(),
                        transaction,
                        fee,
                    });
                }

                routes.push(Route::Eip155(route));
            }
            #[cfg(feature = "solana")]
            EstimatedRouteTransaction::Solana(estimated_transactions) => {
                let mut route =
                    Vec::with_capacity(estimated_transactions.len());
                for (estimated_transaction,) in estimated_transactions {
                    route.push(SolanaTxnDetails {
                        transaction_hash_to_sign: estimated_transaction
                            .transaction
                            .message
                            .serialize()
                            .into(),
                        transaction: estimated_transaction,
                    });
                }
                routes.push(Route::Solana(route));
            }
        }
    }

    let initial_fee = compute_amounts(
        estimated_initial_transaction.2,
        &mut vec![&mut total_local_fee],
        fungibles
            .iter()
            .find(|f| {
                f.address
                    == format!(
                        "{}:{}",
                        estimated_initial_transaction.0.chain_id,
                        NATIVE_TOKEN_ADDRESS.to_checksum(None)
                    )
            })
            .unwrap(),
    );
    let transaction = FeeEstimatedTransaction::from_transaction_and_estimate(
        estimated_initial_transaction.0,
        estimated_initial_transaction.1,
    );
    let initial = TxnDetails {
        transaction_hash_to_sign: transaction.clone().into_signing_hash(),
        transaction,
        fee: initial_fee,
    };

    let mut bridge =
        Vec::with_capacity(route_response.metadata.funding_from.len());
    for item in &route_response.metadata.funding_from {
        let fungible = fungibles
            .iter()
            .find(|f| {
                f.address
                    == format!("{}:{}", item.chain_id, item.token_contract)
            })
            .unwrap();
        if item.symbol != fungible.symbol {
            warn!(
                "Fungible symbol mismatch: item:{} != fungible:{}",
                item.symbol, fungible.symbol
            );
        }
        if item.decimals != fungible.decimals.get() {
            warn!(
                "Fungible decimals mismatch: item:{} != fungible:{}",
                item.decimals, fungible.decimals
            );
        }
        bridge.push(compute_amounts(
            item.bridging_fee,
            &mut vec![&mut total_local_fee, &mut local_bridge_total_acc],
            fungible,
        ))
    }

    let (local_total_fee, local_total_fee_unit) = total_local_fee.compute();
    let (local_route_total_fee, local_route_total_fee_unit) =
        local_route_total_acc.compute();
    let (local_bridge_total_fee, local_bridge_total_fee_unit) =
        local_bridge_total_acc.compute();
    UiFields {
        route_response,
        route: routes,
        local_route_total: Amount::new(
            "USD".to_owned(),
            local_route_total_fee,
            local_route_total_fee_unit,
        ),
        bridge,
        local_bridge_total: Amount::new(
            "USD".to_owned(),
            local_bridge_total_fee,
            local_bridge_total_fee_unit,
        ),
        initial,
        local_total: Amount::new(
            "USD".to_owned(),
            local_total_fee,
            local_total_fee_unit,
        ),
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase", tag = "namespace")]
pub enum RouteSig {
    #[cfg(feature = "eip155")]
    Eip155(Vec<PrimitiveSignature>),
    #[cfg(feature = "solana")]
    Solana(Vec<solana::SolanaSignature>),
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::chain_abstraction::api::prepare::{
            Eip155OrSolanaAddress, FundingMetadata, InitialTransactionMetadata,
            Metadata, Transactions,
        },
        alloy::primitives::{address, bytes, utils::Unit, Address, U64},
        std::iter,
    };

    #[test]
    fn jakub_case() {
        let chain_id_1 = "eip155:8453".to_owned();
        let token_contract_1 =
            address!("833589fCD6eDb6E08f4c7C32D4f71b54bdA02913");
        let chain_id_2 = "eip155:10".to_owned();
        let token_contract_2 =
            address!("0b2C639c533813f4Aa9D7837CAf62653d097Ff85");

        let chain_1_estimated_fees = Eip1559Estimation {
            max_fee_per_gas: 28790247,
            max_priority_fee_per_gas: 999999,
        };
        let chain_2_estimated_fees = Eip1559Estimation {
            max_fee_per_gas: 101529,
            max_priority_fee_per_gas: 99999,
        };

        let initial_transaction = Transaction {
            from: address!("9CAaB7E1D1ad6eaB4d6a7f479Cb8800da551cbc0"),
            to: token_contract_2,
            value: U256::ZERO,
            gas_limit: U64::ZERO,
            input: bytes!("a9059cbb000000000000000000000000228311b83daf3fc9a0d0a46c0b329942fc8cb2ed00000000000000000000000000000000000000000000000000000000001e8480"),
            nonce: U64::ZERO,
            chain_id: chain_id_2.clone(),
        };
        let route_transaction_1 = Transaction {
            from: address!("9CAaB7E1D1ad6eaB4d6a7f479Cb8800da551cbc0"),
            to: token_contract_1,
            value: U256::ZERO,
            gas_limit: U64::from(0xf9e82),
            input: bytes!("095ea7b30000000000000000000000003a23f943181408eac424116af7b7790c94cb97a5000000000000000000000000000000000000000000000000000000000016cd3e"),
            nonce: U64::from(0x29),
            chain_id: chain_id_1.clone(),
        };
        let route_transaction_2 = Transaction {
            from: address!("9CAaB7E1D1ad6eaB4d6a7f479Cb8800da551cbc0"),
            to: address!("3a23F943181408EAC424116Af7b7790c94Cb97a5"),
            value: U256::ZERO,
            gas_limit: U64::from(0xf9e82),
            input: bytes!("0000019b792ebcb9000000000000000000000000000000000000000000000000000000000016cd3e000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000010e10000000000000000000000000000000000000000000000000000000000001b3b00000000000000000000000000000000000000000000000000000000000000020000000000000000000000009caab7e1d1ad6eab4d6a7f479cb8800da551cbc00000000000000000000000009caab7e1d1ad6eab4d6a7f479cb8800da551cbc00000000000000000000000000000000000000000000000000000000000000002000000000000000000000000833589fcd6edb6e08f4c7c32d4f71b54bda029130000000000000000000000000b2c639c533813f4aa9d7837caf62653d097ff850000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000016bc5d000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000675a91e700000000000000000000000000000000000000000000000000000000675ae5edd00dfeeddeadbeef765753be7f7a64d5509974b0d678e1e3149b02f4"),
            nonce: U64::from(0x2a),
            chain_id: chain_id_1.clone(),
        };

        let fields = ui_fields(
            PrepareResponseAvailable {
                orchestration_id: "".to_owned(),
                metadata: Metadata {
                    funding_from: vec![FundingMetadata {
                        chain_id: chain_id_1.clone(),
                        token_contract: Eip155OrSolanaAddress::Eip155(
                            token_contract_1,
                        ),
                        bridging_fee: U256::from(0x71a7),
                        symbol: "USDC".to_owned(),
                        amount: U256::ZERO,
                        decimals: 18,
                    }],
                    initial_transaction: InitialTransactionMetadata {
                        // Data never used in target function
                        transfer_to: Address::ZERO,
                        amount: U256::ZERO,
                        token_contract: token_contract_2,
                        symbol: "UNREACHABLE".to_owned(),
                        decimals: 18,
                    },
                    check_in: 0,
                },
                initial_transaction: initial_transaction.clone(),
                transactions: vec![Transactions::Eip155(vec![
                    route_transaction_1.clone(),
                    route_transaction_2.clone(),
                ])],
            },
            vec![EstimatedRouteTransaction::Eip155(vec![
                (
                    route_transaction_1,
                    chain_1_estimated_fees,
                    U256::from(0x1b085191ebc2_u64),
                ),
                (
                    route_transaction_2,
                    chain_1_estimated_fees,
                    U256::from(0x1b5e96a07aca_u64),
                ),
            ])],
            (
                initial_transaction,
                chain_2_estimated_fees,
                U256::from(0x5f23db29d8_u64),
            ),
            vec![
                FungiblePriceItem {
                    address: format!("{}:{}", chain_id_1, token_contract_1),
                    name: "".to_owned(),
                    symbol: "USDC".to_owned(),
                    icon_url: "".to_owned(),
                    price: 1.,
                    decimals: Unit::new(18).unwrap(),
                },
                FungiblePriceItem {
                    address: format!("{}:{}", chain_id_2, token_contract_2),
                    name: "".to_owned(),
                    symbol: "USDC".to_owned(),
                    icon_url: "".to_owned(),
                    price: 1.,
                    decimals: Unit::new(18).unwrap(),
                },
                FungiblePriceItem {
                    address: format!("{}:{}", chain_id_1, NATIVE_TOKEN_ADDRESS),
                    name: "".to_owned(),
                    symbol: "ETH".to_owned(),
                    icon_url: "".to_owned(),
                    price: 4000.,
                    decimals: Unit::new(18).unwrap(),
                },
                FungiblePriceItem {
                    address: format!("{}:{}", chain_id_2, NATIVE_TOKEN_ADDRESS),
                    name: "".to_owned(),
                    symbol: "ETH".to_owned(),
                    icon_url: "".to_owned(),
                    price: 4000.,
                    decimals: Unit::new(18).unwrap(),
                },
            ],
        );
        println!("fields: {fields:?}");

        assert_eq!(
            fields.route[0].as_eip155().unwrap()[0]
                .transaction
                .max_fee_per_gas
                .to::<u128>(),
            chain_1_estimated_fees.max_fee_per_gas
        );
        assert_eq!(
            fields.route[0].as_eip155().unwrap()[0]
                .transaction
                .max_priority_fee_per_gas
                .to::<u128>(),
            chain_1_estimated_fees.max_priority_fee_per_gas
        );
        assert_eq!(
            fields.route[0].as_eip155().unwrap()[1]
                .transaction
                .max_fee_per_gas
                .to::<u128>(),
            chain_1_estimated_fees.max_fee_per_gas
        );
        assert_eq!(
            fields.route[0].as_eip155().unwrap()[1]
                .transaction
                .max_priority_fee_per_gas
                .to::<u128>(),
            chain_1_estimated_fees.max_priority_fee_per_gas
        );
        assert_eq!(
            fields.initial.transaction.max_fee_per_gas.to::<u128>(),
            chain_2_estimated_fees.max_fee_per_gas
        );
        assert_eq!(
            fields.initial.transaction.max_priority_fee_per_gas.to::<u128>(),
            chain_2_estimated_fees.max_priority_fee_per_gas
        );

        let total_fee = fields.local_total.as_float_inaccurate();
        let combined_fees =
            iter::once(fields.initial.fee.local_fee.as_float_inaccurate())
                .chain(
                    fields
                        .bridge
                        .iter()
                        .map(|f| f.local_fee.as_float_inaccurate()),
                )
                .chain(fields.route.iter().flat_map(|route| {
                    route.as_eip155().unwrap().iter().map(
                        |TxnDetails {
                             fee: TransactionFee { local_fee, .. },
                             ..
                         }| {
                            local_fee.as_float_inaccurate()
                        },
                    )
                }))
                .sum::<f64>();
        println!("total_fee: {total_fee}");
        println!("combined_fees: {combined_fees}");
        let error = (total_fee - combined_fees).abs();
        println!("error: {error}");
        assert!(error < 0.00000000000001);

        let combined_fees_intermediate_totals = [
            fields.initial.fee.local_fee.as_float_inaccurate(),
            fields.local_route_total.as_float_inaccurate(),
            fields.local_bridge_total.as_float_inaccurate(),
        ]
        .iter()
        .sum::<f64>();
        println!("combined_fees_intermediate_totals: {combined_fees_intermediate_totals}");
        let error = (total_fee - combined_fees_intermediate_totals).abs();
        println!("error: {error}");
        assert!(error < 0.00000000000001);

        assert!((fields.local_total.as_float_inaccurate() - 0.24).abs() < 0.01);

        assert_eq!(fields.local_total.formatted_alt, "$0.24");
        assert_eq!(fields.local_route_total.formatted_alt, "$0.24");
        assert_eq!(fields.local_bridge_total.formatted_alt, "<$0.01");
        assert_eq!(fields.initial.fee.local_fee.formatted_alt, "<$0.01");
    }
}
