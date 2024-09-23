use crate::smart_accounts::account_address::AccountAddress;
use alloy::{
    contract::Error as ContractError,
    primitives::{Address, Bytes},
    sol,
    sol_types::{SolCall, SolError},
};
use std::str::FromStr;

sol!(
    #[allow(missing_docs)]
    #[derive(Debug, PartialEq, Eq)]
    error SenderAddressResult(address sender);
    function getSenderAddress(bytes calldata initCode) public;
);

pub struct GetSenderAddress(getSenderAddressCall);

impl GetSenderAddress {
    pub fn new_with_init_code(init_code: Bytes) -> Self {
        Self(getSenderAddressCall { initCode: init_code })
    }

    pub fn new_with_factory_and_factory_data(
        factory: Address,
        factory_data: Bytes,
    ) -> Self {
        let mut init_code = vec![];
        init_code.extend(factory.as_slice());
        init_code.extend(factory_data);

        let init_code: Bytes = init_code.into();

        Self(getSenderAddressCall { initCode: init_code })
    }

    pub fn encode(&self) -> Vec<u8> {
        getSenderAddressCall::abi_encode(&self.0)
    }
}

pub async fn get_sender_address_v07<P, T, N>(
    provider: &P,
    factory: Address,
    factory_data: Bytes,
    entrypoint: super::EntryPointAddress,
) -> eyre::Result<AccountAddress>
where
    T: alloy::contract::private::Transport + ::core::clone::Clone,
    P: alloy::contract::private::Provider<T, N>,
    N: alloy::contract::private::Network,
{
    let init_code: Bytes = {
        let mut init_code = vec![];
        init_code.extend(factory.as_slice());
        init_code.extend(factory_data);
        init_code.into()
    };

    let instance = super::EntryPoint::new(entrypoint.to_address(), provider);

    let call_builder = instance.getSenderAddress(init_code);

    // Note: you may need to static call getSenderAddress() not call() as per
    // the spec. Leaving as-is for now.
    // let call = call_builder.call_raw().await;

    let call: Result<
        crate::entry_point::EntryPoint::getSenderAddressReturn,
        ContractError,
    > = call_builder.call().await;

    if let Err(error) = call {
        println!("Error: {:?}", error);
        match error {
            ContractError::TransportError(transport_error) => {
                println!("transport_error: {:?}", transport_error);

                let error_resp =
                    transport_error.as_error_resp().unwrap().clone();

                println!("error_resp: {:?}", error_resp.clone());

                let code = error_resp.code;
                println!("error_resp_code: {:?}", code);

                let message = error_resp.message.clone();
                println!("error_resp_message: {:?}", message);

                if let Some(error_resp_data) = error_resp.data.clone() {
                    println!("error_resp_data: {:?}", error_resp_data.clone());

                    let hex_value =
                        error_resp_data.get().split("\"").nth(1).unwrap();

                    let hex = hex_value.to_string();

                    let hex = hex.strip_prefix("0x").unwrap();

                    let error_resp_data_bytes_bytes =
                        Bytes::from_str(hex).unwrap();

                    println!(
                        "error_resp_data_bytes_bytes: {:?}",
                        error_resp_data_bytes_bytes.clone()
                    );

                    let decoded_data = SenderAddressResult::abi_decode(
                        &error_resp_data_bytes_bytes,
                        true,
                    )?;

                    let addr = decoded_data.sender;

                    println!("addr: {:?}", addr.clone());

                    return Ok(addr.into());
                } else {
                    return Err(eyre::eyre!("No data in error response"));
                };
            }
            _ => {
                println!("error: {:?}", error);
                todo!()
            }
        }
    };

    Err(eyre::eyre!("Invalid entrypoint"))
}
