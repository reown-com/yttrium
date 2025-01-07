## Current design

```mermaid
sequenceDiagram
    User->>+App: Initiate transaction
    App->>+Wallet: eth_sendTransaction
    Wallet->>+WalletKit: prepare()
    WalletKit->>+Blockchain API RPC: is account deployed?
    Blockchain API RPC->>-WalletKit: No
    Note over WalletKit: Construct 7702 Authorization
    WalletKit->>-Wallet: Request sign Authorization
    Wallet->>+User: Request sign
    User->>-Wallet: Approve
    Wallet->>+WalletKit: deploy()
    Note over WalletKit: Construct SignedAuthorization
    Note over WalletKit: Construct 7702 txn
    Note over WalletKit: Sign by Sponsor EOA
    WalletKit->>+Sponsor EOA: sign()
    Sponsor EOA->>-WalletKit: signature
    WalletKit->>+Blockchain API RPC: eth_sendRawTransaction
    Blockchain API RPC->>-WalletKit: txn receipt
    Note over WalletKit: Account now deployed :tada:
    Note over WalletKit: Construct UserOperation
    WalletKit->>+Blockchain API Paymaster: pimlico_getUserOperationGasPrice
    Blockchain API Paymaster->>-WalletKit: UserOp maxFeePerGas, etc.
    Note over WalletKit: Add maxFeePerGas to UserOperation
    WalletKit->>+Blockchain API Paymaster: pm_sponsorUserOperation
    Blockchain API Paymaster->>-WalletKit: paymaster_data, etc.
    Note over WalletKit: Add paymaster_data to UserOperation
    WalletKit->>-Wallet: Request sign UserOperation
    Wallet->>+User: Request sign UserOperation
    User->>-Wallet: Approve
    Wallet->>+WalletKit: send()
    WalletKit->>+Blockchain API Bundler: eth_sendUserOperation
    Blockchain API Bundler->>-WalletKit: UserOperationReceipt
    WalletKit->>-Wallet: txn receipt
    Wallet->>-App: txn receipt
    App->>-User: Success
```

Sponsor EOA is an EOA that has enough funds to pay for and execute the 7702 txn on the chain.

This diagram assumes current 4337 and paymasters stay the same.

## Ideal

Ideally 4337 and paymasters support 7702 natively and then it would look like:

- TODO: only 2 methods, not 3
- TODO: paymaster pays for 7702 txn
- FIXME: bundlers can't execute `AddSafe7579Contract::addSafe7579Call` because it's not a UserOp?
  - Or maybe this is the factory? :thinking:

Update pain points doc with the above
