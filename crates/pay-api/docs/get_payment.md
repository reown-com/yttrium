# Get Payment Options

Retrieves available payment options for a given payment and set of user accounts.

## Method

`getPayment`

## Request

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `paymentId` | string | Yes | The unique payment identifier (e.g., `wcp_payment_<32 base58 characters>`). |
| `accounts` | string[] | Yes | Array of CAIP-10 account identifiers representing the user's accounts. |

### Example Request

```json
{
  "method": "getPayment",
  "params": {
    "paymentId": "wcp_payment_7XJkF2nPqR9vL5mT3hYwZ6aB4cD8eG1j",
    "accounts": ["eip155:1:0x1234567890abcdef1234567890abcdef12345678", "eip155:137:0x1234567890abcdef1234567890abcdef12345678"]
  }
}
```

## Response

| Field | Type | Description |
|-------|------|-------------|
| TODO | TODO | TODO: Define response fields |

### Example Response

```json
{
  // TODO: Define response structure
}
```

## Errors

| Error | Description |
|-------|-------------|
| TODO | TODO: Define error cases |
