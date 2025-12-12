#[allow(unused_imports)]
pub use progenitor_client::{ByteStream, Error, ResponseValue};
#[allow(unused_imports)]
use progenitor_client::{encode_path, RequestBuilderExt};
#[allow(unused_imports)]
use reqwest::header::{HeaderMap, HeaderValue};
/// Types used as operation parameters and responses.
#[allow(clippy::all)]
pub mod types {
    /// Error types.
    pub mod error {
        /// Error from a TryFrom or FromStr implementation.
        pub struct ConversionError(::std::borrow::Cow<'static, str>);
        impl ::std::error::Error for ConversionError {}
        impl ::std::fmt::Display for ConversionError {
            fn fmt(
                &self,
                f: &mut ::std::fmt::Formatter<'_>,
            ) -> Result<(), ::std::fmt::Error> {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }
        impl ::std::fmt::Debug for ConversionError {
            fn fmt(
                &self,
                f: &mut ::std::fmt::Formatter<'_>,
            ) -> Result<(), ::std::fmt::Error> {
                ::std::fmt::Debug::fmt(&self.0, f)
            }
        }
        impl From<&'static str> for ConversionError {
            fn from(value: &'static str) -> Self {
                Self(value.into())
            }
        }
        impl From<String> for ConversionError {
            fn from(value: String) -> Self {
                Self(value.into())
            }
        }
    }
    ///Amount
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "unit",
    ///    "value"
    ///  ],
    ///  "properties": {
    ///    "unit": {
    ///      "type": "string"
    ///    },
    ///    "value": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug, PartialEq)]
    pub struct Amount {
        pub unit: ::std::string::String,
        pub value: ::std::string::String,
    }
    impl ::std::convert::From<&Amount> for Amount {
        fn from(value: &Amount) -> Self {
            value.clone()
        }
    }
    impl Amount {
        pub fn builder() -> builder::Amount {
            Default::default()
        }
    }
    ///CreatePayment
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "amount",
    ///    "referenceId"
    ///  ],
    ///  "properties": {
    ///    "amount": {
    ///      "$ref": "#/components/schemas/Amount"
    ///    },
    ///    "referenceId": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug, PartialEq)]
    pub struct CreatePayment {
        pub amount: Amount,
        #[serde(rename = "referenceId")]
        pub reference_id: ::std::string::String,
    }
    impl ::std::convert::From<&CreatePayment> for CreatePayment {
        fn from(value: &CreatePayment) -> Self {
            value.clone()
        }
    }
    impl CreatePayment {
        pub fn builder() -> builder::CreatePayment {
            Default::default()
        }
    }
    ///CreatePaymentResponse
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "amount",
    ///    "expiresAt",
    ///    "gatewayUrl",
    ///    "paymentId",
    ///    "pollInMs",
    ///    "status"
    ///  ],
    ///  "properties": {
    ///    "amount": {
    ///      "$ref": "#/components/schemas/Amount"
    ///    },
    ///    "expiresAt": {
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "gatewayUrl": {
    ///      "type": "string"
    ///    },
    ///    "paymentId": {
    ///      "type": "string"
    ///    },
    ///    "pollInMs": {
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "status": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug, PartialEq)]
    pub struct CreatePaymentResponse {
        pub amount: Amount,
        #[serde(rename = "expiresAt")]
        pub expires_at: i64,
        #[serde(rename = "gatewayUrl")]
        pub gateway_url: ::std::string::String,
        #[serde(rename = "paymentId")]
        pub payment_id: ::std::string::String,
        #[serde(rename = "pollInMs")]
        pub poll_in_ms: i64,
        pub status: ::std::string::String,
    }
    impl ::std::convert::From<&CreatePaymentResponse> for CreatePaymentResponse {
        fn from(value: &CreatePaymentResponse) -> Self {
            value.clone()
        }
    }
    impl CreatePaymentResponse {
        pub fn builder() -> builder::CreatePaymentResponse {
            Default::default()
        }
    }
    /// Types for composing complex structures.
    pub mod builder {
        #[derive(Clone, Debug)]
        pub struct Amount {
            unit: ::std::result::Result<::std::string::String, ::std::string::String>,
            value: ::std::result::Result<::std::string::String, ::std::string::String>,
        }
        impl ::std::default::Default for Amount {
            fn default() -> Self {
                Self {
                    unit: Err("no value supplied for unit".to_string()),
                    value: Err("no value supplied for value".to_string()),
                }
            }
        }
        impl Amount {
            pub fn unit<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.unit = value
                    .try_into()
                    .map_err(|e| {
                        format!("error converting supplied value for unit: {}", e)
                    });
                self
            }
            pub fn value<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.value = value
                    .try_into()
                    .map_err(|e| {
                        format!("error converting supplied value for value: {}", e)
                    });
                self
            }
        }
        impl ::std::convert::TryFrom<Amount> for super::Amount {
            type Error = super::error::ConversionError;
            fn try_from(
                value: Amount,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    unit: value.unit?,
                    value: value.value?,
                })
            }
        }
        impl ::std::convert::From<super::Amount> for Amount {
            fn from(value: super::Amount) -> Self {
                Self {
                    unit: Ok(value.unit),
                    value: Ok(value.value),
                }
            }
        }
        #[derive(Clone, Debug)]
        pub struct CreatePayment {
            amount: ::std::result::Result<super::Amount, ::std::string::String>,
            reference_id: ::std::result::Result<
                ::std::string::String,
                ::std::string::String,
            >,
        }
        impl ::std::default::Default for CreatePayment {
            fn default() -> Self {
                Self {
                    amount: Err("no value supplied for amount".to_string()),
                    reference_id: Err("no value supplied for reference_id".to_string()),
                }
            }
        }
        impl CreatePayment {
            pub fn amount<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Amount>,
                T::Error: ::std::fmt::Display,
            {
                self.amount = value
                    .try_into()
                    .map_err(|e| {
                        format!("error converting supplied value for amount: {}", e)
                    });
                self
            }
            pub fn reference_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.reference_id = value
                    .try_into()
                    .map_err(|e| {
                        format!(
                            "error converting supplied value for reference_id: {}", e
                        )
                    });
                self
            }
        }
        impl ::std::convert::TryFrom<CreatePayment> for super::CreatePayment {
            type Error = super::error::ConversionError;
            fn try_from(
                value: CreatePayment,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    amount: value.amount?,
                    reference_id: value.reference_id?,
                })
            }
        }
        impl ::std::convert::From<super::CreatePayment> for CreatePayment {
            fn from(value: super::CreatePayment) -> Self {
                Self {
                    amount: Ok(value.amount),
                    reference_id: Ok(value.reference_id),
                }
            }
        }
        #[derive(Clone, Debug)]
        pub struct CreatePaymentResponse {
            amount: ::std::result::Result<super::Amount, ::std::string::String>,
            expires_at: ::std::result::Result<i64, ::std::string::String>,
            gateway_url: ::std::result::Result<
                ::std::string::String,
                ::std::string::String,
            >,
            payment_id: ::std::result::Result<
                ::std::string::String,
                ::std::string::String,
            >,
            poll_in_ms: ::std::result::Result<i64, ::std::string::String>,
            status: ::std::result::Result<::std::string::String, ::std::string::String>,
        }
        impl ::std::default::Default for CreatePaymentResponse {
            fn default() -> Self {
                Self {
                    amount: Err("no value supplied for amount".to_string()),
                    expires_at: Err("no value supplied for expires_at".to_string()),
                    gateway_url: Err("no value supplied for gateway_url".to_string()),
                    payment_id: Err("no value supplied for payment_id".to_string()),
                    poll_in_ms: Err("no value supplied for poll_in_ms".to_string()),
                    status: Err("no value supplied for status".to_string()),
                }
            }
        }
        impl CreatePaymentResponse {
            pub fn amount<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::Amount>,
                T::Error: ::std::fmt::Display,
            {
                self.amount = value
                    .try_into()
                    .map_err(|e| {
                        format!("error converting supplied value for amount: {}", e)
                    });
                self
            }
            pub fn expires_at<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<i64>,
                T::Error: ::std::fmt::Display,
            {
                self.expires_at = value
                    .try_into()
                    .map_err(|e| {
                        format!("error converting supplied value for expires_at: {}", e)
                    });
                self
            }
            pub fn gateway_url<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.gateway_url = value
                    .try_into()
                    .map_err(|e| {
                        format!("error converting supplied value for gateway_url: {}", e)
                    });
                self
            }
            pub fn payment_id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.payment_id = value
                    .try_into()
                    .map_err(|e| {
                        format!("error converting supplied value for payment_id: {}", e)
                    });
                self
            }
            pub fn poll_in_ms<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<i64>,
                T::Error: ::std::fmt::Display,
            {
                self.poll_in_ms = value
                    .try_into()
                    .map_err(|e| {
                        format!("error converting supplied value for poll_in_ms: {}", e)
                    });
                self
            }
            pub fn status<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.status = value
                    .try_into()
                    .map_err(|e| {
                        format!("error converting supplied value for status: {}", e)
                    });
                self
            }
        }
        impl ::std::convert::TryFrom<CreatePaymentResponse>
        for super::CreatePaymentResponse {
            type Error = super::error::ConversionError;
            fn try_from(
                value: CreatePaymentResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    amount: value.amount?,
                    expires_at: value.expires_at?,
                    gateway_url: value.gateway_url?,
                    payment_id: value.payment_id?,
                    poll_in_ms: value.poll_in_ms?,
                    status: value.status?,
                })
            }
        }
        impl ::std::convert::From<super::CreatePaymentResponse>
        for CreatePaymentResponse {
            fn from(value: super::CreatePaymentResponse) -> Self {
                Self {
                    amount: Ok(value.amount),
                    expires_at: Ok(value.expires_at),
                    gateway_url: Ok(value.gateway_url),
                    payment_id: Ok(value.payment_id),
                    poll_in_ms: Ok(value.poll_in_ms),
                    status: Ok(value.status),
                }
            }
        }
    }
}
#[derive(Clone, Debug)]
/**Client for WalletConnect Pay API

Version: 0.1.0*/
pub struct Client {
    pub(crate) baseurl: String,
    pub(crate) client: reqwest::Client,
}
impl Client {
    /// Create a new client.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new(baseurl: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let client = {
            let dur = std::time::Duration::from_secs(15);
            reqwest::ClientBuilder::new().connect_timeout(dur).timeout(dur)
        };
        #[cfg(target_arch = "wasm32")]
        let client = reqwest::ClientBuilder::new();
        Self::new_with_client(baseurl, client.build().unwrap())
    }
    /// Construct a new client with an existing `reqwest::Client`,
    /// allowing more control over its configuration.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new_with_client(baseurl: &str, client: reqwest::Client) -> Self {
        Self {
            baseurl: baseurl.to_string(),
            client,
        }
    }
    /// Get the base URL to which requests are made.
    pub fn baseurl(&self) -> &String {
        &self.baseurl
    }
    /// Get the internal `reqwest::Client` used to make requests.
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
    /// Get the version of this API.
    ///
    /// This string is pulled directly from the source OpenAPI
    /// document and may be in any format the API selects.
    pub fn api_version(&self) -> &'static str {
        "0.1.0"
    }
}
///Merchant-facing API
pub trait ClientMerchantExt {
    /**Sends a `POST` request to `/v1/merchant/payment`

```ignore
let response = client.create_payment()
    .body(body)
    .send()
    .await;
```*/
    fn create_payment(&self) -> builder::CreatePayment;
}
impl ClientMerchantExt for Client {
    fn create_payment(&self) -> builder::CreatePayment {
        builder::CreatePayment::new(self)
    }
}
/// Types for composing operation parameters.
#[allow(clippy::all)]
pub mod builder {
    use super::types;
    #[allow(unused_imports)]
    use super::{
        encode_path, ByteStream, Error, HeaderMap, HeaderValue, RequestBuilderExt,
        ResponseValue,
    };
    /**Builder for [`ClientMerchantExt::create_payment`]

[`ClientMerchantExt::create_payment`]: super::ClientMerchantExt::create_payment*/
    #[derive(Debug, Clone)]
    pub struct CreatePayment<'a> {
        client: &'a super::Client,
        body: Result<types::builder::CreatePayment, String>,
    }
    impl<'a> CreatePayment<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Ok(::std::default::Default::default()),
            }
        }
        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::CreatePayment>,
            <V as std::convert::TryInto<types::CreatePayment>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| {
                    format!("conversion to `CreatePayment` for body failed: {}", s)
                });
            self
        }
        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(
                types::builder::CreatePayment,
            ) -> types::builder::CreatePayment,
        {
            self.body = self.body.map(f);
            self
        }
        ///Sends a `POST` request to `/v1/merchant/payment`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<types::CreatePaymentResponse>, Error<()>> {
            let Self { client, body } = self;
            let body = body
                .and_then(|v| {
                    types::CreatePayment::try_from(v).map_err(|e| e.to_string())
                })
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/v1/merchant/payment", client.baseurl,);
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .header(
                    reqwest::header::ACCEPT,
                    reqwest::header::HeaderValue::from_static("application/json"),
                )
                .json(&body)
                .build()?;
            let result = client.client.execute(request).await;
            let response = result?;
            match response.status().as_u16() {
                201u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }
}
/// Items consumers will typically use such as the Client and
/// extension traits.
pub mod prelude {
    #[allow(unused_imports)]
    pub use super::Client;
    pub use super::ClientMerchantExt;
}
