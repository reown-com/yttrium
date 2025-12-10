# Envelope Format

All API requests and responses use a consistent envelope format for structured communication.

## Request Envelope

Requests use a tagged union format with `method` identifying the operation and `params` containing the method-specific payload.

| Field | Type | Description |
|-------|------|-------------|
| `method` | string | The API method to invoke. |
| `params` | object | Method-specific parameters. |

### Available Methods

| Method | Description | Consumer |
|--------|-------------|----------|
| [`createPayment`](./create_payment.md) | Create a new payment intent. | POS / PSP |
| [`getPaymentStatus`](./get_payment_status.md) | Poll for payment status updates. | POS / PSP |
| [`getPayment`](./get_payment.md) | Retrieve payment details and options. | Wallets |
| `confirmPayment` | Execute payment (blocking, returns final status). | Wallets |
| `buildPaymentRequest` | Build a payment transaction request. | Internal |

### Example Request

```json
{
  "method": "createPayment",
  "params": {
    "referenceId": "ORDER-456",
    "amount": {
      "unit": "iso4217/USD",
      "value": "1000"
    }
  }
}
```

## Response Envelope

Responses use a tagged union format with `status` indicating success or error.

### Success Response

| Field | Type | Description |
|-------|------|-------------|
| `status` | `"success"` | Indicates successful operation. |
| `data` | object | Method-specific response data. |

### Example Success Response

```json
{
  "status": "success",
  "data": {
    "paymentId": "wcp_payment_7XJkF2nPqR9vL5mT3hYwZ6aB4cD8eG1j",
    "status": "requires_action",
    "amount": {
      "unit": "iso4217/USD",
      "value": "1000"
    },
    "expiresAt": 1733126400,
    "pollInMs": 1000
  }
}
```

### Error Response

| Field | Type | Description |
|-------|------|-------------|
| `status` | `"error"` | Indicates failed operation. |
| `error` | object | Error details. |
| `error.code` | string | Machine-readable error code. |
| `error.message` | string | Human-readable error description. |

### Example Error Response

```json
{
  "status": "error",
  "error": {
    "code": "INVALID_PARAMS",
    "message": "Amount cannot be zero"
  }
}
```

## Error Codes

| Code | Description |
|------|-------------|
| `INTERNAL_ERROR` | An unexpected server error occurred. Retryable. |
| `METHOD_NOT_FOUND` | The requested method does not exist. |
| `INVALID_PARAMS` | The request parameters are malformed or missing. |
| `PARAMS_VALIDATION` | The parameters failed validation rules. |

## Headers

| Header | Required | Description |
|--------|----------|-------------|
| `x-api-key` | Yes | Merchant API key for authentication. |
| `idempotency-key` | No | Unique key to ensure idempotent requests. |

