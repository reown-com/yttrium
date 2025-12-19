use crate::{
    bodies::{
        create_payment::{Amount, CreatePayment, CreatePaymentResponse},
        get_payment_status::{
            GetPaymentStatusParams, GetPaymentStatusResponse,
        },
    },
    envelope::{ErrorResponse, GatewayRequest, GatewayResponse},
};

#[test]
fn test_create_payment_request_serialize() {
    let input = GatewayRequest::CreatePayment(CreatePayment {
        reference_id: "ORDER-456".to_string(),
        amount: Amount {
            unit: "iso4217/USD".to_string(),
            value: "1000".to_string(),
        },
    });
    let result = serde_json::to_value(input).unwrap();
    assert_eq!(
        result,
        serde_json::json!({
            "method": "createPayment",
            "params": {
                "referenceId": "ORDER-456",
                "amount": {
                    "unit": "iso4217/USD",
                    "value": "1000",
                },
            },
        })
    );
}

#[test]
fn test_create_payment_request_deserialize() {
    let input = serde_json::json!({
        "method": "createPayment",
        "params": {
            "referenceId": "ORDER-456",
            "amount": {
                "unit": "iso4217/USD",
                "value": "1000",
            },
        },
    });
    let result = serde_json::from_value::<GatewayRequest>(input).unwrap();
    assert!(matches!(result, GatewayRequest::CreatePayment(_)));
    let GatewayRequest::CreatePayment(request) = result else {
        panic!("Expected CreatePayment request");
    };
    assert_eq!(
        request.amount,
        Amount { unit: "iso4217/USD".to_string(), value: "1000".to_string() }
    );
    assert_eq!(request.reference_id, "ORDER-456");
}

#[test]
fn test_create_payment_response_success() {
    let input = GatewayResponse::Success {
        data: CreatePaymentResponse {
            payment_id: "pay_123".to_string(),
            status: "requires_action".to_string(),
            amount: Amount {
                unit: "iso4217/USD".to_string(),
                value: "1000".to_string(),
            },
            expires_at: 1733000000,
            poll_in_ms: 1000,
            gateway_url: "https://walletconnect.com/pay".to_string(),
        },
    };
    let expected = serde_json::json!({
        "status": "success",
        "data": {
            "paymentId": "pay_123",
            "status": "requires_action",
            "gatewayUrl": "https://walletconnect.com/pay",
            "amount": {
                "unit": "iso4217/USD",
                "value": "1000",
            },
            "expiresAt": 1733000000,
            "pollInMs": 1000,
            "gatewayUrl": "https://pay.walletconnect.com/wcp_payment_7XJkF2nPqR9vL5mT3hYwZ6aB4cD8eG1j",
        },
    });
    assert_eq!(serde_json::to_value(input).unwrap(), expected);
}

#[test]
fn test_create_payment_response_error() {
    let input = GatewayResponse::<()>::Error {
        error: ErrorResponse {
            code: "123".to_string(),
            message: "error".to_string(),
        },
    };
    let expected = serde_json::json!({
        "status": "error",
        "error": {
            "code": "123",
            "message": "error",
        },
    });
    assert_eq!(serde_json::to_value(input).unwrap(), expected);
}

#[test]
fn test_get_payment_status_request_serialize() {
    let input = GatewayRequest::GetPaymentStatus(GetPaymentStatusParams {
        payment_id: "pay_123".to_string(),
    });
    let result = serde_json::to_value(input).unwrap();
    assert_eq!(
        result,
        serde_json::json!({
            "method": "getPaymentStatus",
            "params": {
                "paymentId": "pay_123",
            },
        })
    );
}

#[test]
fn test_get_payment_status_request_deserialize() {
    let input = serde_json::json!({
        "method": "getPaymentStatus",
        "params": {
            "paymentId": "pay_123",
        },
    });
    let result = serde_json::from_value::<GatewayRequest>(input).unwrap();
    let GatewayRequest::GetPaymentStatus(params) = result else {
        panic!("Expected GetPaymentStatus request");
    };
    assert_eq!(params.payment_id, "pay_123");
}

#[test]
fn test_get_payment_status_response_success() {
    let input = GatewayResponse::Success {
        data: GetPaymentStatusResponse {
            payment_id: "pay_123".to_string(),
            status: "requires_action".to_string(),
            poll_in_ms: 1000,
        },
    };
    let expected = serde_json::json!({
        "status": "success",
        "data": {
            "paymentId": "pay_123",
            "status": "requires_action",
            "pollInMs": 1000,
        },
    });
    assert_eq!(serde_json::to_value(input).unwrap(), expected);
}
