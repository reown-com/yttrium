use {
    alloy::{
        primitives::Bytes,
        rpc::types::UserOperationReceipt,
        transports::{TransportError, TransportResult},
    },
    alloy_provider::Provider,
    serde_json::Value,
};

pub async fn get_user_operation_receipt(
    provider: &impl Provider,
    hash: Bytes,
) -> TransportResult<Option<UserOperationReceipt>> {
    provider
        .get_user_operation_receipt(hash)
        .await?
        .map(map_user_operation_receipt)
        .transpose()
}

pub fn map_user_operation_receipt(
    mut value: Value,
) -> TransportResult<UserOperationReceipt> {
    // For some reason Pimlico bundler doesn't include these fields
    // Workaround by injecting them in

    if let Some(value) = value.as_object_mut() {
        value.get_mut("receipt").map(|receipt| {
            receipt.as_object_mut().map(|receipt| {
                receipt.entry("type").or_insert_with(|| {
                    serde_json::Value::String("0x0".to_owned())
                })
            })
        });

        value
            .entry("reason")
            .or_insert(serde_json::Value::String("".to_owned()));
        value.entry("paymaster").or_insert(serde_json::Value::String(
            "0x0000000000000000000000000000000000000000".to_owned(),
        ));
    }

    serde_json::from_value::<UserOperationReceipt>(value).map_err(|e| {
        TransportError::deser_err(
            e,
            "Failed to deserialize UserOperationReceipt",
        )
    })
}

// Private (for now) so that it forces folks to use Pimlico-compatible `get_user_operation_receipt` above
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
trait CustomErc4337Api: Send + Sync {
    async fn get_user_operation_receipt(
        &self,
        user_op_hash: Bytes,
    ) -> TransportResult<Option<Value>>;
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl<P> CustomErc4337Api for P
where
    P: Provider,
{
    async fn get_user_operation_receipt(
        &self,
        user_op_hash: Bytes,
    ) -> TransportResult<Option<Value>> {
        self.client()
            .request("eth_getUserOperationReceipt", (user_op_hash,))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let json = serde_json::json!({
            "logs": [
                {
                    "data": "0xa6097513f3e0b3d6b76b0971724cb5e140541d6627dbdaf72248325034b9b34400000000000000000000000000000000002b0ecfbd0496ee71e01257da0e37de0000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b33",
                    "topics": [
                        "0x5d14f8bf6f75758495bb0b0768b81cdebc7869d1f19edacc2f483ca0c89a1715"
                    ],
                    "address": "0x0000003111cD8e92337C100F22B7A9dbf8DEE301",
                    "logIndex": "0x96",
                    "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                    "blockNumber": "0x17c84ed",
                    "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                    "transactionIndex": "0xc"
                },
                {
                    "data": "0xe1e770a1c3ab7d5ac27ffc745d13f43cd2bf687fa2366232bf86b4a85d75cbd300000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000003111cd8e92337c100f22b7a9dbf8dee3010000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b33",
                    "topics": [
                        "0x1c5ea73ef804151087de6d0ce7a77287b15cfc4f51654cac5bfe6c558547f827"
                    ],
                    "address": "0x00000000002B0eCfbD0496EE71e01257dA0E37DE",
                    "logIndex": "0x97",
                    "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                    "blockNumber": "0x17c84ed",
                    "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                    "transactionIndex": "0xc"
                },
                {
                    "data": "0x13002086fc3966c6773f60586f871e96879e565c1056d27d980e808ef02c2a1400000000000000000000000000000000002b0ecfbd0496ee71e01257da0e37de0000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b33",
                    "topics": [
                        "0x5d14f8bf6f75758495bb0b0768b81cdebc7869d1f19edacc2f483ca0c89a1715"
                    ],
                    "address": "0x0000003111cD8e92337C100F22B7A9dbf8DEE301",
                    "logIndex": "0x98",
                    "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                    "blockNumber": "0x17c84ed",
                    "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                    "transactionIndex": "0xc"
                },
                {
                    "data": "0xe1e770a1c3ab7d5ac27ffc745d13f43cd2bf687fa2366232bf86b4a85d75cbd300000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000003111cd8e92337c100f22b7a9dbf8dee3010000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b33",
                    "topics": [
                        "0x1c5ea73ef804151087de6d0ce7a77287b15cfc4f51654cac5bfe6c558547f827"
                    ],
                    "address": "0x00000000002B0eCfbD0496EE71e01257dA0E37DE",
                    "logIndex": "0x99",
                    "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                    "blockNumber": "0x17c84ed",
                    "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                    "transactionIndex": "0xc"
                },
                {
                    "data": "0xe1e770a1c3ab7d5ac27ffc745d13f43cd2bf687fa2366232bf86b4a85d75cbd30000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b330000000000000000000000000000000000000000000000000000000000000001",
                    "topics": [
                        "0x2c479309476f30bc996bdf6bdea053f2a125e86e9f080ee5a87c1eff76220937"
                    ],
                    "address": "0x00000000002B0eCfbD0496EE71e01257dA0E37DE",
                    "logIndex": "0x9a",
                    "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                    "blockNumber": "0x17c84ed",
                    "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                    "transactionIndex": "0xc"
                },
                {
                    "data": "0xe1e770a1c3ab7d5ac27ffc745d13f43cd2bf687fa2366232bf86b4a85d75cbd3000000000000000000000000caf0461410340f8f366f1f7f7716cf1d90b6bda40000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b33",
                    "topics": [
                        "0x9e1deb1dcdedd38e68d5543a9a6be8416a0e0a5627628e47780bd6ef8abc1a3c"
                    ],
                    "address": "0x00000000002B0eCfbD0496EE71e01257dA0E37DE",
                    "logIndex": "0x9b",
                    "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                    "blockNumber": "0x17c84ed",
                    "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                    "transactionIndex": "0xc"
                },
                {
                    "data": "0x",
                    "topics": [
                        "0x6895c13664aa4f67288b25d7a21d7aaa34916e355fb9b6fae0a139a9085becb8",
                        "0x0000000000000000000000007579ee8307284f293b1927136486880611f20002"
                    ],
                    "address": "0x5b1Eb68657A65E340F19036AfC5FD92057c29B33",
                    "logIndex": "0x9c",
                    "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                    "blockNumber": "0x17c84ed",
                    "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                    "transactionIndex": "0xc"
                },
                {
                    "data": "0x0000000000000000000000000000000000000000000000000000016701a90fec",
                    "topics": [
                        "0x2da466a7b24304f47e87fa2e1e5a81b9831ce54fec19055ce277ca2f39ba42c4",
                        "0x0000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b33"
                    ],
                    "address": "0x0000000071727De22E5E9d8BAf0edAc6f37da032",
                    "logIndex": "0x9d",
                    "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                    "blockNumber": "0x17c84ed",
                    "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                    "transactionIndex": "0xc"
                },
                {
                    "data": "0x",
                    "topics": [
                        "0x6895c13664aa4f67288b25d7a21d7aaa34916e355fb9b6fae0a139a9085becb8",
                        "0x0000000000000000000000007579ee8307284f293b1927136486880611f20002"
                    ],
                    "address": "0x5b1Eb68657A65E340F19036AfC5FD92057c29B33",
                    "logIndex": "0x9e",
                    "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                    "blockNumber": "0x17c84ed",
                    "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                    "transactionIndex": "0xc"
                },
                {
                    "data": "0x",
                    "topics": [
                        "0xbb47ee3e183a558b1a2ff0874b079f3fc5478b7454eacf2bfc5af2ff5878f972"
                    ],
                    "address": "0x0000000071727De22E5E9d8BAf0edAc6f37da032",
                    "logIndex": "0x9f",
                    "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                    "blockNumber": "0x17c84ed",
                    "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                    "transactionIndex": "0xc"
                },
                {
                    "data": "0x",
                    "topics": [
                        "0x6895c13664aa4f67288b25d7a21d7aaa34916e355fb9b6fae0a139a9085becb8",
                        "0x0000000000000000000000007579ee8307284f293b1927136486880611f20002"
                    ],
                    "address": "0x5b1Eb68657A65E340F19036AfC5FD92057c29B33",
                    "logIndex": "0xa0",
                    "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                    "blockNumber": "0x17c84ed",
                    "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                    "transactionIndex": "0xc"
                }
            ],
            "nonce": "0x2b0ecfbd0496ee71e01257da0e37de000000000000000000000011",
            "sender": "0x5b1Eb68657A65E340F19036AfC5FD92057c29B33",
            "receipt": {
                "to": "0x0000000071727De22E5E9d8BAf0edAc6f37da032",
                "from": "0x433700890211c1C776C391D414Cffd38efdd1811",
                "logs": [
                    {
                        "data": "0xa6097513f3e0b3d6b76b0971724cb5e140541d6627dbdaf72248325034b9b34400000000000000000000000000000000002b0ecfbd0496ee71e01257da0e37de0000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b33",
                        "topics": [
                            "0x5d14f8bf6f75758495bb0b0768b81cdebc7869d1f19edacc2f483ca0c89a1715"
                        ],
                        "address": "0x0000003111cD8e92337C100F22B7A9dbf8DEE301",
                        "logIndex": "0x96",
                        "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                        "blockNumber": "0x17c84ed",
                        "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                        "transactionIndex": "0xc"
                    },
                    {
                        "data": "0xe1e770a1c3ab7d5ac27ffc745d13f43cd2bf687fa2366232bf86b4a85d75cbd300000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000003111cd8e92337c100f22b7a9dbf8dee3010000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b33",
                        "topics": [
                            "0x1c5ea73ef804151087de6d0ce7a77287b15cfc4f51654cac5bfe6c558547f827"
                        ],
                        "address": "0x00000000002B0eCfbD0496EE71e01257dA0E37DE",
                        "logIndex": "0x97",
                        "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                        "blockNumber": "0x17c84ed",
                        "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                        "transactionIndex": "0xc"
                    },
                    {
                        "data": "0x13002086fc3966c6773f60586f871e96879e565c1056d27d980e808ef02c2a1400000000000000000000000000000000002b0ecfbd0496ee71e01257da0e37de0000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b33",
                        "topics": [
                            "0x5d14f8bf6f75758495bb0b0768b81cdebc7869d1f19edacc2f483ca0c89a1715"
                        ],
                        "address": "0x0000003111cD8e92337C100F22B7A9dbf8DEE301",
                        "logIndex": "0x98",
                        "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                        "blockNumber": "0x17c84ed",
                        "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                        "transactionIndex": "0xc"
                    },
                    {
                    "data": "0xe1e770a1c3ab7d5ac27ffc745d13f43cd2bf687fa2366232bf86b4a85d75cbd300000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000003111cd8e92337c100f22b7a9dbf8dee3010000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b33",
                        "topics": [
                            "0x1c5ea73ef804151087de6d0ce7a77287b15cfc4f51654cac5bfe6c558547f827"
                        ],
                        "address": "0x00000000002B0eCfbD0496EE71e01257dA0E37DE",
                        "logIndex": "0x99",
                        "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                        "blockNumber": "0x17c84ed",
                        "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                        "transactionIndex": "0xc"
                    },
                    {
                        "data": "0xe1e770a1c3ab7d5ac27ffc745d13f43cd2bf687fa2366232bf86b4a85d75cbd30000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b330000000000000000000000000000000000000000000000000000000000000001",
                        "topics": [
                            "0x2c479309476f30bc996bdf6bdea053f2a125e86e9f080ee5a87c1eff76220937"
                        ],
                        "address": "0x00000000002B0eCfbD0496EE71e01257dA0E37DE",
                        "logIndex": "0x9a",
                        "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                        "blockNumber": "0x17c84ed",
                        "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                        "transactionIndex": "0xc"
                    },
                    {
                        "data": "0xe1e770a1c3ab7d5ac27ffc745d13f43cd2bf687fa2366232bf86b4a85d75cbd3000000000000000000000000caf0461410340f8f366f1f7f7716cf1d90b6bda40000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b33",
                        "topics": [
                            "0x9e1deb1dcdedd38e68d5543a9a6be8416a0e0a5627628e47780bd6ef8abc1a3c"
                        ],
                        "address": "0x00000000002B0eCfbD0496EE71e01257dA0E37DE",
                        "logIndex": "0x9b",
                        "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                        "blockNumber": "0x17c84ed",
                        "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                        "transactionIndex": "0xc"
                    },
                    {
                        "data": "0x",
                        "topics": [
                            "0x6895c13664aa4f67288b25d7a21d7aaa34916e355fb9b6fae0a139a9085becb8",
                            "0x0000000000000000000000007579ee8307284f293b1927136486880611f20002"
                        ],
                        "address": "0x5b1Eb68657A65E340F19036AfC5FD92057c29B33",
                        "logIndex": "0x9c",
                        "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                        "blockNumber": "0x17c84ed",
                        "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                        "transactionIndex": "0xc"
                    },
                    {
                        "data": "0x0000000000000000000000000000000000000000000000000000016701a90fec",
                        "topics": [
                            "0x2da466a7b24304f47e87fa2e1e5a81b9831ce54fec19055ce277ca2f39ba42c4",
                            "0x0000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b33"
                        ],
                        "address": "0x0000000071727De22E5E9d8BAf0edAc6f37da032",
                        "logIndex": "0x9d",
                        "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                        "blockNumber": "0x17c84ed",
                        "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                        "transactionIndex": "0xc"
                    },
                    {
                        "data": "0x",
                        "topics": [
                            "0x6895c13664aa4f67288b25d7a21d7aaa34916e355fb9b6fae0a139a9085becb8",
                            "0x0000000000000000000000007579ee8307284f293b1927136486880611f20002"
                        ],
                        "address": "0x5b1Eb68657A65E340F19036AfC5FD92057c29B33",
                        "logIndex": "0x9e",
                        "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                        "blockNumber": "0x17c84ed",
                        "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                        "transactionIndex": "0xc"
                    },
                    {
                        "data": "0x",
                        "topics": [
                            "0xbb47ee3e183a558b1a2ff0874b079f3fc5478b7454eacf2bfc5af2ff5878f972"
                        ],
                        "address": "0x0000000071727De22E5E9d8BAf0edAc6f37da032",
                        "logIndex": "0x9f",
                        "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                        "blockNumber": "0x17c84ed",
                        "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                        "transactionIndex": "0xc"
                    },
                    {
                        "data": "0x",
                        "topics": [
                            "0x6895c13664aa4f67288b25d7a21d7aaa34916e355fb9b6fae0a139a9085becb8",
                            "0x0000000000000000000000007579ee8307284f293b1927136486880611f20002"
                        ],
                        "address": "0x5b1Eb68657A65E340F19036AfC5FD92057c29B33",
                        "logIndex": "0xa0",
                        "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                        "blockNumber": "0x17c84ed",
                        "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                        "transactionIndex": "0xc"
                    },
                    {
                        "data": "0x00000000002b0ecfbd0496ee71e01257da0e37de00000000000000000000001100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000010e7f76a46600000000000000000000000000000000000000000000000000000000000bbe8f",
                        "topics": [
                            "0x49628fd1471006c1482da88028e9ce4dbb080b815c9b0344d39e5a8e6ec1419f",
                            "0x3669aec5be67df7ebac851088040a79b1d2d5cee63c7089bfc329208f8caaf7f",
                            "0x0000000000000000000000005b1eb68657a65e340f19036afc5fd92057c29b33",
                            "0x0000000000000000000000000000000000000000000000000000000000000000"
                        ],
                        "address": "0x0000000071727De22E5E9d8BAf0edAc6f37da032",
                        "logIndex": "0xa1",
                        "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                        "blockNumber": "0x17c84ed",
                        "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                        "transactionIndex": "0xc"
                    }
                ],
                "status": "0x1",
                "gasUsed": "0xb864d",
                "blockHash": "0xc79878eb43ccf9fd7160f25235e6c7ce48d96abbb9bb9b645db16ca5b3167565",
                "logsBloom": "0x0000100000001000004010000000000000000000000000000000000000000000000c000000000000000001010200100200000000000000000000020000000000000000000000000000000010100000000040081000008000000000000000200080000000020800000000000000000800000000000000000000000000000202000002000800000020000000000000000000000000000000000000000000000000000008200000000000408000000000000208000000000000000002000010000000000000000000400401000000000000000000000000000000000000000020000040000000000400200000800000000200002000000000400008000000000000",
                "blockNumber": "0x17c84ed",
                "contractAddress": null,
                "transactionHash": "0xb9faea20466c459556521943d073f2f5d0306e0f2f5793e703a509aea99d1885",
                "transactionIndex": "0xc",
                "cumulativeGasUsed": "0x479eba",
                "effectiveGasPrice": "0x14f8e2"
            },
            "success": true,
            "entryPoint": "0x0000000071727de22e5e9d8baf0edac6f37da032",
            "userOpHash": "0x3669aec5be67df7ebac851088040a79b1d2d5cee63c7089bfc329208f8caaf7f",
            "actualGasCost": "0x10e7f76a466",
            "actualGasUsed": "0xbbe8f"
        });
        let receipt = map_user_operation_receipt(json).unwrap();
        println!("{:?}", receipt);
    }
}
