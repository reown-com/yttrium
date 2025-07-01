use alloy::{rpc::client::RpcClient, transports::TransportResult};

pub struct StacksProvider {
    pub client: RpcClient,
}

impl StacksProvider {
    pub async fn stacks_transactions(
        &self,
        tx_hex: String,
    ) -> TransportResult<serde_json::Value> {
        // TODO proper return type
        let response: serde_json::Value =
            match self.client.request("stacks_transactions", tx_hex).await {
                Ok(result) => result,
                Err(e) => {
                    return Err(e);
                }
            };

        Ok(response)
    }

    // Queries a proxy method on blockchain API which queries `/v2/accounts/<Principal>` on Stacks API https://docs.stacks.co/reference/api#get-v2-accounts-principal
    pub async fn stacks_accounts(
        &self,
        principal: String,
    ) -> TransportResult<serde_json::Value> {
        let response: serde_json::Value =
            match self.client.request("stacks_accounts", principal).await {
                Ok(result) => result,
                Err(e) => {
                    return Err(e);
                }
            };

        Ok(response)
    }

    // // Queries a proxy method on blockchain API which queries /v2/fees/transaction
    // pub async fn hiro_fees_transaction(
    //     &self,
    //     transaction_payload: String,
    // ) -> TransportResult<serde_json::Value> {
    //     // Query the current fee rate from the Stacks network
    //     // The fee is typically around 180 microSTX, but we'll query it dynamically
    //     let response: serde_json::Value = match self
    //         .client
    //         .request("hiro_fees_transaction", transaction_payload)
    //         .await
    //     {
    //         Ok(result) => result,
    //         Err(e) => {
    //             return Err(e);
    //         }
    //     };

    //     Ok(response)
    // }

    // Queries a proxy method on blockchain API which queries /v2/fees/transaction
    pub async fn stacks_transfer_fees(&self) -> TransportResult<serde_json::Value> {
        // Query the current fee rate from the Stacks network
        // The fee is typically around 180 microSTX, but we'll query it dynamically
        let response: serde_json::Value =
            match self.client.request("stacks_transfer_fees", "{}").await {
                Ok(result) => result,
                Err(e) => {
                    return Err(e);
                }
            };

        Ok(response)
    }

    // // Queries a proxy method on blockchain API which queries `/extended/v1/address/<principal>/nonces` endpoint on Hiro API https://docs.hiro.so/stacks/api/accounts/latest-nonce
    // pub async fn stacks_extended_nonces(
    //     &self,
    //     principal: String,
    // ) -> TransportResult<u64> {
    //     let response: serde_json::Value = match self
    //         .client
    //         .request("stacks_extended_nonces", principal)
    //         .await
    //     {
    //         Ok(result) => result,
    //         Err(e) => {
    //             return Err(e);
    //         }
    //     };

    //     let possible_next_nonce = response
    //         .get("possible_next_nonce")
    //         .and_then(|v| v.as_u64())
    //         .unwrap_or(0);

    //     Ok(possible_next_nonce)
    // }
}
