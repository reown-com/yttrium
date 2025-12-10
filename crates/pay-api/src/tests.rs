use crate::{
    bodies::{
        create_payment::{Amount, CreatePayment, CreatePaymentResponse},
        get_payment::{GetPaymentParams, GetPaymentResponse, PaymentOption, TokenInfo},
        get_payment_status::{GetPaymentStatusParams, GetPaymentStatusResponse},
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
        Amount {
            unit: "iso4217/USD".to_string(),
            value: "1000".to_string(),
        }
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
        },
    };
    let expected = serde_json::json!({
        "status": "success",
        "data": {
            "paymentId": "pay_123",
            "status": "requires_action",
            "amount": {
                "unit": "iso4217/USD",
                "value": "1000",
            },
            "expiresAt": 1733000000,
            "pollInMs": 1000,
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
fn test_get_payment_request_serialize() {
    let input = GatewayRequest::GetPayment(GetPaymentParams {
        payment_id: "pay_123".to_string(),
        accounts: vec!["0x123".to_string()],
    });
    let result = serde_json::to_value(input).unwrap();
    assert_eq!(
        result,
        serde_json::json!({
            "method": "getPayment",
            "params": {
                "paymentId": "pay_123",
                "accounts": ["0x123"],
            },
        })
    );
}

#[test]
fn test_get_payment_request_deserialize_with_accounts() {
    let input = serde_json::json!({
        "method": "getPayment",
        "params": {
            "paymentId": "pay_123",
            "accounts": ["0x123"],
        },
    });
    let result = serde_json::from_value::<GatewayRequest>(input).unwrap();
    let GatewayRequest::GetPayment(params) = result else {
        panic!("Expected GetPayment request");
    };
    assert_eq!(params.payment_id, "pay_123");
    assert_eq!(params.accounts, vec!["0x123"]);
}

#[test]
fn test_get_payment_response_success() {
    let input = GatewayResponse::Success {
        data: GetPaymentResponse {
            payment_id: "pay_123".to_string(),
            status: "requires_action".to_string(),
            amount: Amount {
                unit: "iso4217/USD".to_string(),
                value: "1000".to_string(),
            },
            options: vec![],
        },
    };
    let expected = serde_json::json!({
        "status": "success",
        "data": {
            "paymentId": "pay_123",
            "status": "requires_action",
            "amount": {
                "unit": "iso4217/USD",
                "value": "1000",
            },
            "options": [],
        },
    });
    assert_eq!(serde_json::to_value(input).unwrap(), expected);
}

#[test]
fn test_get_payment_response_with_options() {
    let input = GatewayResponse::Success {
        data: GetPaymentResponse {
            payment_id: "pay_123".to_string(),
            status: "requires_action".to_string(),
            amount: Amount {
                unit: "iso4217/USD".to_string(),
                value: "1000".to_string(),
            },
            options: vec![PaymentOption {
                option_id: "opt_abc123".to_string(),
                chain_id: "eip155:1".to_string(),
                token: TokenInfo {
                    address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                },
                amount: "10000000".to_string(),
                fee: "100000".to_string(),
            }],
        },
    };
    let expected = serde_json::json!({
        "status": "success",
        "data": {
            "paymentId": "pay_123",
            "status": "requires_action",
            "amount": {
                "unit": "iso4217/USD",
                "value": "1000",
            },
            "options": [{
                "optionId": "opt_abc123",
                "chainId": "eip155:1",
                "token": {
                    "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                    "symbol": "USDC",
                    "decimals": 6,
                },
                "amount": "10000000",
                "fee": "100000",
            }],
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
