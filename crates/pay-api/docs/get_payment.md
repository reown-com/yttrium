# Get Payment

Retrieves payment details and available payment options for a wallet user. Used by wallets to display payment options to the user.

## Method

`getPayment`

## Consumer

**Wallets** - For fetching payment details and available payment options.

> **Note**: POS/PSP systems should use [`getPaymentStatus`](./get_payment_status.md) for polling payment status (simpler response without options).

## Request

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `paymentId` | string | Yes | The payment identifier from the payment link or QR code. |
| `accounts` | string[] | Yes | List of wallet accounts in CAIP-10 format (e.g., `eip155:1:0x...`). Used to compute available payment options. |

### Example Request

```json
{
  "paymentId": "wcp_payment_7XJkF2nPqR9vL5mT3hYwZ6aB4cD8eG1j",
  "accounts": [
    "eip155:1:0x1234567890abcdef1234567890abcdef12345678",
    "eip155:137:0x1234567890abcdef1234567890abcdef12345678"
  ]
}
```

## Response

| Field | Type | Description |
|-------|------|-------------|
| `paymentId` | string | The payment identifier. |
| `status` | string | Current payment status. See [Payment Statuses](#payment-statuses). |
| `amount` | object | Requested payment amount. |
| `amount.unit` | string | Currency identifier (e.g., `iso4217/USD`). |
| `amount.value` | string | Amount in minor units. |
| `options` | array | Available payment options computed based on provided accounts. Empty array until options computation is implemented. |

### Example Response

```json
{
  "paymentId": "wcp_payment_7XJkF2nPqR9vL5mT3hYwZ6aB4cD8eG1j",
  "status": "requires_action",
  "amount": {
    "unit": "iso4217/USD",
    "value": "1000"
  },
  "options": [
    {
      "optionId": "opt_abc123",
      "chainId": "eip155:1",
      "token": {
        "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "symbol": "USDC",
        "decimals": 6
      },
      "amount": "10000000",
      "fee": "100000"
    }
  ]
}
```

## Payment Statuses

| Status | Description |
|--------|-------------|
| `requires_action` | Waiting for user to select a payment option and confirm. |
| `processing` | Payment transaction submitted, awaiting confirmation. |
| `succeeded` | Payment completed successfully. |
| `failed` | Payment failed (e.g., transaction reverted). |
| `expired` | Payment expired before completion. |

## Payment Options

Each option in the `options` array represents a way the user can pay:

| Field | Type | Description |
|-------|------|-------------|
| `optionId` | string | Unique identifier for this payment option. Pass to `confirmPayment`. |
| `chainId` | string | CAIP-2 chain identifier (e.g., `eip155:1` for Ethereum mainnet). |
| `token` | object | Token details. |
| `token.address` | string | Token contract address. |
| `token.symbol` | string | Token symbol (e.g., `USDC`). |
| `token.decimals` | number | Token decimals. |
| `amount` | string | Amount to transfer in token's smallest unit. |
| `fee` | string | Fee amount in token's smallest unit. |

## Wallet Flow

1. User scans QR code or opens payment link containing `paymentId`
2. Wallet calls `getPayment` with user's connected accounts
3. Wallet displays payment options to user
4. User selects an option
5. Wallet calls `confirmPayment` with selected `optionId`
6. `confirmPayment` blocks and returns final status (no polling needed)

## Errors

| Error | Description |
|-------|-------------|
| `Payment not found` | No payment exists with the specified `paymentId`. |
| `Accounts required` | The `accounts` array is required for wallet flow. |
