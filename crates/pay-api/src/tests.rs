use crate::{
    bodies::create_payment::{CreatePayment, CreatePaymentResponse},
    envelope::{ErrorResponse, GatewayRequest, GatewayResponse},
};

#[test]
fn test_create_payment_request_serialize() {
    let input = GatewayRequest::CreatePayment(CreatePayment {
        amount: "100".to_string(),
        currency: "iso4217/USD".to_string(),
        reference_id: "123".to_string(),
    });
    let result = serde_json::to_value(input).unwrap();
    assert_eq!(
        result,
        serde_json::json!({
            "method": "createPayment",
            "params": {
                "amount": "100",
                "currency": "iso4217/USD",
                "referenceId": "123",
            },
        })
    );
}

#[test]
fn test_create_payment_request_deserialize() {
    let input = serde_json::json!({
        "method": "createPayment",
        "params": {
            "amount": "100",
            "currency": "iso4217/USD",
            "referenceId": "123",
        },
    });
    let result = serde_json::from_value::<GatewayRequest>(input).unwrap();
    assert!(matches!(result, GatewayRequest::CreatePayment(_)));
    let GatewayRequest::CreatePayment(request) = result else {
        panic!("Expected CreatePayment request");
    };
    assert_eq!(request.amount, "100");
    assert_eq!(request.currency, "iso4217/USD");
    assert_eq!(request.reference_id, "123");
}

#[test]
fn test_create_payment_response_success() {
    let input = GatewayResponse::Success {
        data: CreatePaymentResponse { payment_id: "123".to_string() },
    };
    let expected = serde_json::json!({
        "status": "success",
        "data": {
            "paymentId": "123",
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
