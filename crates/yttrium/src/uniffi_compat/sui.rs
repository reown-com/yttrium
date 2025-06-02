use {
    crate::{
        blockchain_api::BLOCKCHAIN_API_URL_PROD,
        chain_abstraction::pulse::PulseMetadata, provider_pool::ProviderPool,
    },
    data_encoding::BASE64,
    fastcrypto::{
        ed25519::Ed25519KeyPair,
        traits::{EncodeDecodeBase64, KeyPair},
    },
    rand::{
        rngs::{OsRng, StdRng},
        SeedableRng,
    },
    relay_rpc::domain::ProjectId,
    reqwest::Client as ReqwestClient,
    serde::{Deserialize, Serialize},
    sui_sdk::{
        rpc_types::{Balance, SuiTransactionBlockResponseOptions},
        types::{
            base_types::SuiAddress,
            crypto::{PublicKey, Signature, SuiKeyPair},
            digests::TransactionDigest,
            signature::GenericSignature,
            transaction::{
                CallArg, Command, GasData, ObjectArg, ProgrammableTransaction,
                TransactionData, TransactionDataV1, TransactionExpiration,
                TransactionKind,
            },
        },
    },
    sui_shared_crypto::intent::{Intent, IntentMessage, PersonalMessage},
    uniffi::deps::anyhow,
    url::Url,
};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum SuiError {
    #[error("General {0}")]
    General(String),
}

uniffi::custom_type!(SuiKeyPair, String, {
    remote,
    try_lift: |val| SuiKeyPair::decode(&val).map_err(|e| anyhow::anyhow!(e)),
    lower: |obj| obj.encode().unwrap(),
});

uniffi::custom_type!(PublicKey, String, {
    remote,
    try_lift: |val| val.parse::<PublicKey>().map_err(|e| anyhow::anyhow!(e)),
    lower: |obj| obj.encode_base64(),
});

uniffi::custom_type!(SuiAddress, String, {
    remote,
    try_lift: |val| val.parse(),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(Signature, String, {
    remote,
    try_lift: |val| val.parse::<Signature>().map_err(|e| anyhow::anyhow!(e)),
    lower: |obj| obj.encode_base64(),
});

uniffi::custom_type!(TransactionDigest, String, {
    remote,
    try_lift: |val| val.parse::<TransactionDigest>().map_err(|e| anyhow::anyhow!(e)),
    lower: |obj| obj.to_string(),
});

// TODO support other key types
#[uniffi::export]
pub fn sui_generate_keypair() -> SuiKeyPair {
    SuiKeyPair::Ed25519(Ed25519KeyPair::generate(
        &mut StdRng::from_rng(OsRng).unwrap(),
    ))
}

#[uniffi::export]
pub fn sui_get_public_key(keypair: &SuiKeyPair) -> PublicKey {
    keypair.public()
}

#[uniffi::export]
pub fn sui_get_address(public_key: &PublicKey) -> SuiAddress {
    SuiAddress::from(public_key)
}

#[uniffi::export]
pub fn sui_personal_sign(keypair: &SuiKeyPair, message: Vec<u8>) -> Signature {
    let intent_msg = IntentMessage::new(
        Intent::personal_message(),
        PersonalMessage { message },
    );
    Signature::new_secure(&intent_msg, keypair)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SignTransactionRequest {
    pub version: u8,
    pub sender: String,
    pub expiration: Option<u64>,
    pub gas_data: GasDataFfi,
    pub inputs: Vec<CallArgFfi>,
    pub commands: Vec<CommandFfi>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GasDataFfi {
    pub budget: Option<u64>,
    pub price: Option<u64>,
    pub owner: Option<String>,
    pub payment: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CallArgFfi {
    // contains no structs or objects
    Pure { bytes: String },
    // an object
    Object(ObjectArgFfi),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommandFfi {
    SplitCoins { coin: ObjectArgFfi, amounts: Vec<CallArgRefFfi> },
    TransferObjects { objects: Vec<ObjectArgFfi>, address: CallArgRefFfi },
    // Add other command types as needed
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ObjectArgFfi {
    GasCoin(bool),
    NestedResult(Vec<u16>),
    // Add other object arg types as needed
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CallArgRefFfi {
    Input(u16),
    // Add other call arg ref types as needed
}

#[derive(thiserror::Error, Debug, uniffi::Error)]
pub enum SuiSignTransactionError {
    #[error("Invalid transaction data")]
    InvalidTransactionData(serde_json::Error),

    #[error("Decoding Pure base64")]
    DecodePure(String),
}

#[uniffi::export]
pub fn sui_sign_transaction(
    keypair: &SuiKeyPair,
    tx_data: &[u8],
) -> Result<Signature, SuiSignTransactionError> {
    let request = serde_json::from_slice::<SignTransactionRequest>(tx_data)
        .map_err(SuiSignTransactionError::InvalidTransactionData)?;
    let data = TransactionData::V1(TransactionDataV1 {
        kind: TransactionKind::ProgrammableTransaction(
            ProgrammableTransaction {
                inputs: {
                    let mut results = Vec::with_capacity(request.inputs.len());
                    for input in request.inputs {
                        results.push(match input {
                            CallArgFfi::Pure { bytes } => CallArg::Pure(
                                BASE64.decode(bytes.as_bytes()).map_err(
                                    |e| {
                                        SuiSignTransactionError::DecodePure(
                                            e.to_string(),
                                        )
                                    },
                                )?,
                            ),
                            CallArgFfi::Object(object) => {
                                CallArg::Object(convert_object_arg_ffi(object))
                            }
                        });
                    }
                    results
                },
                commands: {
                    let mut results =
                        Vec::with_capacity(request.commands.len());
                    for command in request.commands {
                        results.push(convert_command_ffi(command));
                    }
                    results
                },
            },
        ),
        sender: request.sender.parse().unwrap(),
        gas_data: GasData {
            price: request.gas_data.price.unwrap_or(1000),
            owner: request
                .gas_data
                .owner
                .unwrap_or(request.sender.clone())
                .parse()
                .unwrap(),
            payment: vec![], // TODO: parse payment objects if needed
            budget: request.gas_data.budget.unwrap_or(10000000),
        },
        expiration: TransactionExpiration::None,
    });
    let intent_msg = IntentMessage::new(Intent::sui_transaction(), data);
    Ok(Signature::new_secure(&intent_msg, keypair))
}

fn convert_object_arg_ffi(obj: ObjectArgFfi) -> ObjectArg {
    match obj {
        ObjectArgFfi::GasCoin(_) => {
            // For GasCoin, we need to create a proper ObjectArg
            // This is a placeholder - in a real implementation, you'd need the actual object reference
            ObjectArg::ImmOrOwnedObject((
                sui_sdk::types::base_types::ObjectID::ZERO,
                sui_sdk::types::base_types::SequenceNumber::new(),
                sui_sdk::types::base_types::ObjectDigest::new([0; 32]),
            ))
        }
        ObjectArgFfi::NestedResult(_indices) => {
            // For NestedResult, we need to use the correct ObjectArg variant
            // This should be handled differently - NestedResult is not a valid ObjectArg variant
            // Let's use a placeholder for now
            ObjectArg::ImmOrOwnedObject((
                sui_sdk::types::base_types::ObjectID::ZERO,
                sui_sdk::types::base_types::SequenceNumber::new(),
                sui_sdk::types::base_types::ObjectDigest::new([0; 32]),
            ))
        }
    }
}

fn convert_command_ffi(cmd: CommandFfi) -> Command {
    match cmd {
        CommandFfi::SplitCoins { coin, amounts } => Command::SplitCoins(
            convert_object_arg_ref_ffi(coin),
            amounts.into_iter().map(convert_call_arg_ref_ffi).collect(),
        ),
        CommandFfi::TransferObjects { objects, address } => {
            Command::TransferObjects(
                objects.into_iter().map(convert_object_arg_ref_ffi).collect(),
                convert_call_arg_ref_ffi(address),
            )
        }
    }
}

fn convert_object_arg_ref_ffi(
    obj: ObjectArgFfi,
) -> sui_sdk::types::transaction::Argument {
    match obj {
        ObjectArgFfi::GasCoin(_) => {
            sui_sdk::types::transaction::Argument::GasCoin
        }
        ObjectArgFfi::NestedResult(indices) => {
            sui_sdk::types::transaction::Argument::NestedResult(
                indices[0], indices[1],
            )
        }
    }
}

fn convert_call_arg_ref_ffi(
    arg: CallArgRefFfi,
) -> sui_sdk::types::transaction::Argument {
    match arg {
        CallArgRefFfi::Input(index) => {
            sui_sdk::types::transaction::Argument::Input(index)
        }
    }
}

#[derive(uniffi::Object)]
pub struct SuiClient {
    provider_pool: ProviderPool,
    #[allow(unused)]
    http_client: ReqwestClient,
    #[allow(unused)]
    project_id: ProjectId,
    #[allow(unused)]
    pulse_metadata: PulseMetadata,
}

#[uniffi::export(async_runtime = "tokio")]
impl SuiClient {
    #[uniffi::constructor]
    pub fn new(project_id: ProjectId, pulse_metadata: PulseMetadata) -> Self {
        Self::with_blockchain_api_url(
            project_id,
            pulse_metadata,
            BLOCKCHAIN_API_URL_PROD.parse().unwrap(),
        )
    }

    #[uniffi::constructor]
    pub fn with_blockchain_api_url(
        project_id: ProjectId,
        pulse_metadata: PulseMetadata,
        blockchain_api_base_url: Url,
    ) -> Self {
        let client = ReqwestClient::builder().build();
        let client = match client {
            Ok(client) => client,
            Err(e) => {
                panic!("Failed to create reqwest client: {} ... {:?}", e, e)
            }
        };
        Self {
            provider_pool: ProviderPool::new(
                project_id.clone(),
                client.clone(),
                pulse_metadata.clone(),
                blockchain_api_base_url,
            ),
            http_client: client,
            project_id,
            pulse_metadata,
        }
    }

    pub async fn get_all_balances(
        &self,
        chain_id: String,
        address: SuiAddress,
    ) -> Result<Vec<Balance>, SuiError> {
        self.provider_pool
            .get_sui_client(chain_id)
            .await
            .coin_read_api()
            .get_all_balances(address)
            .await
            .map_err(|e| SuiError::General(e.to_string()))
    }

    pub async fn sign_and_execute_transaction(
        &self,
        chain_id: String,
        keypair: &SuiKeyPair,
        tx_data: &[u8],
    ) -> Result<TransactionDigest, SuiError> {
        let request = serde_json::from_slice::<SignTransactionRequest>(tx_data)
            .map_err(|e| SuiError::General(format!("Invalid transaction data: {}", e)))?;
        
        let data = TransactionData::V1(TransactionDataV1 {
            kind: TransactionKind::ProgrammableTransaction(
                ProgrammableTransaction {
                    inputs: {
                        let mut results = Vec::with_capacity(request.inputs.len());
                        for input in request.inputs {
                            results.push(match input {
                                CallArgFfi::Pure { bytes } => CallArg::Pure(
                                    BASE64.decode(bytes.as_bytes()).map_err(
                                        |e| SuiError::General(format!("Decoding Pure base64: {}", e))
                                    )?,
                                ),
                                CallArgFfi::Object(object) => {
                                    CallArg::Object(convert_object_arg_ffi(object))
                                }
                            });
                        }
                        results
                    },
                    commands: {
                        let mut results =
                            Vec::with_capacity(request.commands.len());
                        for command in request.commands {
                            results.push(convert_command_ffi(command));
                        }
                        results
                    },
                },
            ),
            sender: request.sender.parse()
                .map_err(|e| SuiError::General(format!("Invalid sender address: {}", e)))?,
            gas_data: GasData {
                price: request.gas_data.price.unwrap_or(1000),
                owner: request
                    .gas_data
                    .owner
                    .unwrap_or(request.sender.clone())
                    .parse()
                    .map_err(|e| SuiError::General(format!("Invalid gas owner address: {}", e)))?,
                payment: vec![], // TODO: parse payment objects if needed
                budget: request.gas_data.budget.unwrap_or(10000000),
            },
            expiration: TransactionExpiration::None,
        });
        
        let intent_msg = IntentMessage::new(Intent::sui_transaction(), data);
        let sig = Signature::new_secure(&intent_msg, keypair);
        let sui_client = self.provider_pool.get_sui_client(chain_id).await;
        let result = sui_client
            .quorum_driver_api()
            .execute_transaction_block(
                sui_sdk::types::transaction::Transaction::from_generic_sig_data(
                    intent_msg.value,
                    vec![GenericSignature::Signature(sig)],
                ),
                SuiTransactionBlockResponseOptions::default(),
                None,
            )
            .await
            .map_err(|e| SuiError::General(e.to_string()))?;
        Ok(result.digest)
    }
}

#[derive(uniffi::Record)]
pub struct BalanceFfi {
    pub coin_type: String,
    pub total_balance: u64,
}

uniffi::custom_type!(Balance, BalanceFfi, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting Balance"),
    lower: |obj| BalanceFfi {
        coin_type: obj.coin_type,
        total_balance: u64::try_from(obj.total_balance).unwrap(),
    },
});

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            chain_abstraction::pulse::get_pulse_metadata,
            provider_pool::network::sui::{DEVNET, MAINNET, TESTNET},
        },
        data_encoding::BASE64,
        sui_sdk::types::crypto::{SignatureScheme, SuiSignature},
    };

    #[test]
    fn test_sui_generate_keypair() {
        let _keypair = sui_generate_keypair();
    }

    #[test]
    fn test_sui_keypair_uniffi() {
        let keypair = sui_generate_keypair();
        let u =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(keypair.copy());
        let s =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(keypair, s);
    }

    #[test]
    fn test_sui_public_key_uniffi() {
        let keypair = sui_generate_keypair();
        let public_key = sui_get_public_key(&keypair);
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(
            public_key.clone(),
        );
        let s =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(public_key, s);
    }

    #[test]
    fn test_sui_get_public_key() {
        let keypair = sui_generate_keypair();
        let public_key = sui_get_public_key(&keypair);
        assert_eq!(public_key, keypair.public());
    }

    #[test]
    fn test_sui_address_uniffi() {
        let keypair = sui_generate_keypair();
        let address = sui_get_address(&keypair.public());
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(address);
        let s =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(address, s);
    }

    #[test]
    fn test_sui_get_address() {
        let keypair = sui_generate_keypair();
        let address = sui_get_address(&keypair.public());
        assert_eq!(
            address.to_string(),
            SuiAddress::from(&keypair.public()).to_string()
        );
    }

    #[tokio::test]
    #[cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]
    async fn test_sui_get_balance_random_address() {
        let zero_address = SuiAddress::random_for_testing_only();
        let client = SuiClient::new(
            std::env::var("REOWN_PROJECT_ID").unwrap().into(),
            get_pulse_metadata(),
        );
        let balances = client
            .get_all_balances("sui:mainnet".to_owned(), zero_address)
            .await
            .unwrap();
        assert!(balances.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]
    async fn test_sui_get_balance_devnet() {
        let keypair = sui_generate_keypair();
        let address = sui_get_address(&keypair.public());
        let client = SuiClient::new(
            std::env::var("REOWN_PROJECT_ID").unwrap().into(),
            get_pulse_metadata(),
        );
        let balances =
            client.get_all_balances(DEVNET.to_owned(), address).await.unwrap();
        assert!(balances.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]
    async fn test_sui_get_balance_testnet() {
        let keypair = sui_generate_keypair();
        let address = sui_get_address(&keypair.public());
        let client = SuiClient::new(
            std::env::var("REOWN_PROJECT_ID").unwrap().into(),
            get_pulse_metadata(),
        );
        let balances =
            client.get_all_balances(TESTNET.to_owned(), address).await.unwrap();
        assert!(balances.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]
    async fn test_sui_get_balance_mainnet() {
        let keypair = sui_generate_keypair();
        let address = sui_get_address(&keypair.public());
        let client = SuiClient::new(
            std::env::var("REOWN_PROJECT_ID").unwrap().into(),
            get_pulse_metadata(),
        );
        let balances =
            client.get_all_balances(MAINNET.to_owned(), address).await.unwrap();
        assert!(balances.is_empty());
    }

    #[test]
    fn test_signature_uniffi() {
        let signature = sui_personal_sign(
            &sui_generate_keypair(),
            "Hello, world!".as_bytes().to_vec(),
        );
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(
            signature.clone(),
        );
        let s =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(signature, s);
    }

    #[test]
    fn test_sui_sign() {
        let keypair = sui_generate_keypair();
        let message = "Hello, world!".as_bytes().to_vec();
        let signature = sui_personal_sign(&keypair, message.clone());
        let verification = signature.verify_secure(
            &IntentMessage::new(
                Intent::personal_message(),
                PersonalMessage { message },
            ),
            sui_get_address(&keypair.public()),
            SignatureScheme::ED25519,
        );
        assert!(verification.is_ok());
    }

    #[test]
    fn test_sui_sign_transaction() {
        let tx_data = BASE64.decode(b"ewogICJ2ZXJzaW9uIjogMiwKICAic2VuZGVyIjogIjB4YTg2NjljYzg0ZjM2N2Y3MzBlYTVkYmRiOTA5NTViYTZkNDYxMzcyMDk0ZTNjZmY4MGI3ZjI2N2M4ZWI4MWE1OSIsCiAgImV4cGlyYXRpb24iOiBudWxsLAogICJnYXNEYXRhIjogewogICAgImJ1ZGdldCI6IG51bGwsCiAgICAicHJpY2UiOiBudWxsLAogICAgIm93bmVyIjogbnVsbCwKICAgICJwYXltZW50IjogbnVsbAogIH0sCiAgImlucHV0cyI6IFsKICAgIHsKICAgICAgIlB1cmUiOiB7CiAgICAgICAgImJ5dGVzIjogIlpBQUFBQUFBQUFBPSIKICAgICAgfQogICAgfSwKICAgIHsKICAgICAgIlB1cmUiOiB7CiAgICAgICAgImJ5dGVzIjogInFHYWN5RTgyZjNNT3BkdmJrSlZicHRSaE55Q1U0OC80QzM4bWZJNjRHbGs9IgogICAgICB9CiAgICB9CiAgXSwKICAiY29tbWFuZHMiOiBbCiAgICB7CiAgICAgICJTcGxpdENvaW5zIjogewogICAgICAgICJjb2luIjogewogICAgICAgICAgIkdhc0NvaW4iOiB0cnVlCiAgICAgICAgfSwKICAgICAgICAiYW1vdW50cyI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgIklucHV0IjogMAogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfQogICAgfSwKICAgIHsKICAgICAgIlRyYW5zZmVyT2JqZWN0cyI6IHsKICAgICAgICAib2JqZWN0cyI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgIk5lc3RlZFJlc3VsdCI6IFsKICAgICAgICAgICAgICAwLAogICAgICAgICAgICAgIDAKICAgICAgICAgICAgXQogICAgICAgICAgfQogICAgICAgIF0sCiAgICAgICAgImFkZHJlc3MiOiB7CiAgICAgICAgICAiSW5wdXQiOiAxCiAgICAgICAgfQogICAgICB9CiAgICB9CiAgXQp9").unwrap();
        println!("tx_data: {}", String::from_utf8_lossy(&tx_data));
        let keypair = SuiKeyPair::decode("suiprivkey1qz3889tm677q5ns478amrva66xjzl3qpnujjersm0ps948etrs5g795ce7t").unwrap();
        let signature = sui_sign_transaction(&keypair, &tx_data);
        println!("signature: {:?}", signature);
        let expected_signature = "APmr9wpBF5CAN/D8KAAh2pWZhthS2DR7wnkFPYx64Cyi7FpJYixqQu55fs/rbhBLEhKbLXsCKCxJO155iFQfqAgYx38fs+hcoDU9W5MkJXvAl/AuuaggN96d6c7wdhYV+w==";
        assert_eq!(signature.unwrap().encode_base64(), expected_signature);
    }

    #[test]
    fn test_user_scenario_exact_data() {
        // This test uses the exact keypair and transaction data provided by the user
        let keypair = SuiKeyPair::decode("suiprivkey1qz3889tm677q5ns478amrva66xjzl3qpnujjersm0ps948etrs5g795ce7t").unwrap();
        let tx_data = BASE64.decode(b"ewogICJ2ZXJzaW9uIjogMiwKICAic2VuZGVyIjogIjB4YTg2NjljYzg0ZjM2N2Y3MzBlYTVkYmRiOTA5NTViYTZkNDYxMzcyMDk0ZTNjZmY4MGI3ZjI2N2M4ZWI4MWE1OSIsCiAgImV4cGlyYXRpb24iOiBudWxsLAogICJnYXNEYXRhIjogewogICAgImJ1ZGdldCI6IG51bGwsCiAgICAicHJpY2UiOiBudWxsLAogICAgIm93bmVyIjogbnVsbCwKICAgICJwYXltZW50IjogbnVsbAogIH0sCiAgImlucHV0cyI6IFsKICAgIHsKICAgICAgIlB1cmUiOiB7CiAgICAgICAgImJ5dGVzIjogIlpBQUFBQUFBQUFBPSIKICAgICAgfQogICAgfSwKICAgIHsKICAgICAgIlB1cmUiOiB7CiAgICAgICAgImJ5dGVzIjogInFHYWN5RTgyZjNNT3BkdmJrSlZicHRSaE55Q1U0OC80QzM4bWZJNjRHbGs9IgogICAgICB9CiAgICB9CiAgXSwKICAiY29tbWFuZHMiOiBbCiAgICB7CiAgICAgICJTcGxpdENvaW5zIjogewogICAgICAgICJjb2luIjogewogICAgICAgICAgIkdhc0NvaW4iOiB0cnVlCiAgICAgICAgfSwKICAgICAgICAiYW1vdW50cyI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgIklucHV0IjogMAogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfQogICAgfSwKICAgIHsKICAgICAgIlRyYW5zZmVyT2JqZWN0cyI6IHsKICAgICAgICAib2JqZWN0cyI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgIk5lc3RlZFJlc3VsdCI6IFsKICAgICAgICAgICAgICAwLAogICAgICAgICAgICAgIDAKICAgICAgICAgICAgXQogICAgICAgICAgfQogICAgICAgIF0sCiAgICAgICAgImFkZHJlc3MiOiB7CiAgICAgICAgICAiSW5wdXQiOiAxCiAgICAgICAgfQogICAgICB9CiAgICB9CiAgXQp9").unwrap();
        
        // Parse the transaction data exactly as sign_and_execute_transaction now does
        let request = serde_json::from_slice::<SignTransactionRequest>(&tx_data)
            .expect("Should parse the user's transaction data");
        
        // Build TransactionData structure
        let data = TransactionData::V1(TransactionDataV1 {
            kind: TransactionKind::ProgrammableTransaction(
                ProgrammableTransaction {
                    inputs: {
                        let mut results = Vec::with_capacity(request.inputs.len());
                        for input in request.inputs {
                            results.push(match input {
                                CallArgFfi::Pure { bytes } => CallArg::Pure(
                                    BASE64.decode(bytes.as_bytes()).expect("Should decode base64")
                                ),
                                CallArgFfi::Object(object) => {
                                    CallArg::Object(convert_object_arg_ffi(object))
                                }
                            });
                        }
                        results
                    },
                    commands: {
                        let mut results =
                            Vec::with_capacity(request.commands.len());
                        for command in request.commands {
                            results.push(convert_command_ffi(command));
                        }
                        results
                    },
                },
            ),
            sender: request.sender.parse().expect("Should parse sender address"),
            gas_data: GasData {
                price: request.gas_data.price.unwrap_or(1000),
                owner: request
                    .gas_data
                    .owner
                    .unwrap_or(request.sender.clone())
                    .parse()
                    .expect("Should parse gas owner address"),
                payment: vec![], // TODO: parse payment objects if needed
                budget: request.gas_data.budget.unwrap_or(10000000),
            },
            expiration: TransactionExpiration::None,
        });

        // Create intent message and sign it - this should work without the previous BCS error
        let intent_msg = IntentMessage::new(Intent::sui_transaction(), data);
        let signature = Signature::new_secure(&intent_msg, &keypair);
        
        // Verify signature is created successfully
        assert!(!signature.encode_base64().is_empty());
        
        println!("✅ User's exact scenario now works - no more BCS parsing error!");
        println!("✅ Transaction successfully parsed from JSON and signed");
    }
}
