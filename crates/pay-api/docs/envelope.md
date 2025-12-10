# Envelope Format

All API requests and responses use a consistent envelope format for structured communication.

## Request Envelope

Requests use a tagged union format with `method` identifying the operation and `params` containing the method-specific payload.

| Field | Type | Description |
|-------|------|-------------|
| `method` | string | The API method to invoke. |
| `params` | object | Method-specific parameters. |

### Available Methods

| Method | Description |
|--------|-------------|
| [`createPayment`](./create_payment.md) | Create a new payment intent. |
| `getPayment` | Retrieve payment details. |
| `buildPaymentRequest` | Build a payment transaction request. |
| `confirmPayment` | Confirm a completed payment. |

### Example Request

```json
{
  "method": "createPayment",
  "params": {
    "amount": "1000",
    "currency": "iso4217/USD",
    "referenceId": "order-123"
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
    "paymentId": "wcp_payment_7XJkF2nPqR9vL5mT3hYwZ6aB4cD8eG1j"
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

