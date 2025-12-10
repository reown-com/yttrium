# Create Payment

Creates a new payment intent.

## Method

`createPayment`

## Request

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `referenceId` | string | No | Custom merchant reference ID. Max 35 characters. Only letters, digits, spaces, and `/ - : . , +` allowed. |
| `amount` | object | Yes | Payment amount details. |
| `amount.unit` | string | Yes | Currency identifier in ISO 4217 format prefixed with `iso4217/` (e.g., `iso4217/USD`). |
| `amount.value` | string | Yes | Payment amount as a string in minor units (e.g., cents for USD). Must be a positive integer without decimals or leading zeros. Max 100 characters. |

### Example Request

```json
{
  "referenceId": "ORDER-456",
  "amount": {
    "unit": "iso4217/USD",
    "value": "1000"
  }
}
```

## Response

| Field | Type | Description |
|-------|------|-------------|
| `paymentId` | string | Unique payment identifier in the format `wcp_payment_<32 base58 characters>`. |
| `status` | string | Current payment status. Always `requires_action` for new payments. |
| `amount` | object | Payment amount details. |
| `amount.unit` | string | Currency identifier (e.g., `iso4217/USD`). |
| `amount.value` | string | Amount in minor units. |
| `expiresAt` | number | Unix timestamp (seconds) when the payment expires. |
| `pollInMs` | number | Recommended polling interval in milliseconds. |

### Example Response

```json
{
  "paymentId": "wcp_payment_7XJkF2nPqR9vL5mT3hYwZ6aB4cD8eG1j",
  "status": "requires_action",
  "amount": {
    "unit": "iso4217/USD",
    "value": "1000"
  },
  "expiresAt": 1733126400,
  "pollInMs": 1000
}
```

## Validation Rules

### Amount
- Cannot be empty
- Max 100 characters
- Only digits allowed (no decimals)
- Cannot be zero
- Cannot have leading zeros

### Currency
- Must be a supported currency
- Currently supported: `iso4217/USD`

### Reference ID
- Optional (can be empty)
- Max 35 characters
- Only letters (A-Z, a-z), digits (0-9), spaces, and `/ - : . , +` allowed

## Errors

| Error | Description |
|-------|-------------|
| `Amount is empty` | The amount field was not provided or is empty. |
| `Amount exceeds maximum length of 100 characters` | The amount string is too long. |
| `Amount contains invalid characters, only digits allowed` | The amount contains non-digit characters. |
| `Amount cannot be zero` | The amount is zero. |
| `Amount cannot have leading zeros` | The amount has leading zeros (e.g., "0123"). |
| `Currency '<currency>' is not supported` | The specified currency is not supported. |
| `Reference ID exceeds maximum length of 35 characters` | The reference ID is too long. |
| `Reference ID contains invalid characters` | The reference ID contains characters other than letters, digits, spaces, or `/ - : . , +`. |

## Payment States

A newly created payment starts in the `requires_action` state, indicating it needs user action to proceed with the payment flow.

