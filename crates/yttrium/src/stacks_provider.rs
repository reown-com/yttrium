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
            self.client.request("stacks_transactions", tx_hex).await?;

        println!("Transactions response: {}", response);

        Ok(response)
    }

    // TODO waiting Max to enable these endpoints on backend side so we can query /v2/fees/transfer
    pub async fn estimate_fee(&self) -> TransportResult<u64> {
        // Query the current fee rate from the Stacks network
        // The fee is typically around 180 microSTX, but we'll query it dynamically

        let response: serde_json::Value =
            self.client.request("get_fee_rate", ()).await?;
        println!("Fee estimation response: {:?}", response);

        // Extract the fee rate from the response
        // The response format depends on the Stacks RPC endpoint
        let fee_rate =
            response.get("fee_rate").and_then(|v| v.as_u64()).unwrap_or(180); // Default fallback fee

        println!("Estimated fee rate: {} microSTX", fee_rate);

        Ok(fee_rate)
    }

    // TODO waiting Max to enable these endpoints on backend side so we can query /v2/accounts/[Principal]
    pub async fn get_account_balance(
        &self,
        address: &str,
    ) -> TransportResult<u64> {
        // Query the STX balance for the given address
        let response: serde_json::Value =
            self.client.request("get_account", (address,)).await?;

        // Extract the balance from the response
        let balance = response
            .get("balance")
            .and_then(|v| v.get("stx"))
            .and_then(|v| v.get("balance"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        Ok(balance)
    }
}
