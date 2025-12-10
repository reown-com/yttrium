# Get Payment Status

Retrieves the current status of a payment. Used by POS/PSP systems to poll for payment completion after `createPayment`.

## Method

`getPaymentStatus`

## Consumer

**POS / PSP systems** - For polling payment status updates.

> **Note**: Wallets should use [`getPayment`](./get_payment.md) to retrieve payment options, and [`confirmPayment`](./confirm_payment.md) which returns the final status directly without polling.

## Request

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `paymentId` | string | Yes | The payment identifier returned from `createPayment`. |

### Example Request

```json
{
  "paymentId": "wcp_payment_7XJkF2nPqR9vL5mT3hYwZ6aB4cD8eG1j"
}
```

## Response

| Field | Type | Description |
|-------|------|-------------|
| `paymentId` | string | The payment identifier. |
| `status` | string | Current payment status. See [Payment Statuses](#payment-statuses). |
| `pollInMs` | number | Recommended polling interval in milliseconds. `0` for final states. |

### Example Response (Pending)

```json
{
  "paymentId": "wcp_payment_7XJkF2nPqR9vL5mT3hYwZ6aB4cD8eG1j",
  "status": "requires_action",
  "pollInMs": 1000
}
```

### Example Response (Completed)

```json
{
  "paymentId": "wcp_payment_7XJkF2nPqR9vL5mT3hYwZ6aB4cD8eG1j",
  "status": "succeeded",
  "pollInMs": 0
}
```

## Payment Statuses

| Status | Description | `pollInMs` | Final? |
|--------|-------------|------------|--------|
| `requires_action` | Waiting for user to complete payment in wallet. | 1000 | No |
| `processing` | Payment transaction submitted, awaiting confirmation. | 1000 | No |
| `succeeded` | Payment completed successfully. | 0 | Yes |
| `failed` | Payment failed (e.g., transaction reverted). | 0 | Yes |
| `expired` | Payment expired before completion. | 0 | Yes |

## Polling Behavior

The `pollInMs` field indicates how frequently the client should poll:

- **Non-zero value**: Payment is still in progress. Poll again after the specified interval.
- **Zero**: Payment has reached a final state. No further polling needed.

### Recommended Polling Pattern

```
1. Call createPayment, receive paymentId
2. Call getPaymentStatus with paymentId
3. If pollInMs > 0, wait pollInMs milliseconds
4. Repeat from step 2
5. If pollInMs == 0, display final status to user
```

## Errors

| Error | Description |
|-------|-------------|
| `Payment not found` | No payment exists with the specified `paymentId`. |
