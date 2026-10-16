//! The JSON facade: one string contract for React Native and the web. Method names are the typed
//! API's; params and results are JSON objects with camelCase fields. Errors are
//! `{ code, message }`.

use {
    crate::{
        SignClient, SignError,
        client::types::{
            Namespaces, ProposalParams, RpcError, RpcRequest, SessionEventData,
            embedded_json,
        },
        protocol::{
            ijson,
            types::{DappIntent, Mode},
        },
        rt::boxed,
    },
    serde::{Deserialize, de::DeserializeOwned},
    serde_json::{Value, json},
};

fn args<T: DeserializeOwned>(p: &Value) -> Result<T, SignError> {
    serde_json::from_value(p.clone())
        .map_err(|e| SignError::new("invalid_params", e.to_string()))
}

fn to_json<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).expect("json")
}

pub fn error_json(e: &SignError) -> String {
    let SignError::Sdk { code, message } = e;
    json!({ "code": code, "message": message }).to_string()
}

#[derive(Deserialize)]
struct Topic {
    topic: String,
}

#[derive(Deserialize)]
struct Connect {
    params: ProposalParams,
    intent: Option<DappIntent>,
}

#[derive(Deserialize)]
struct Pair {
    uri: String,
}

#[derive(Deserialize)]
struct Approve {
    topic: String,
    namespaces: Namespaces,
    own: Option<Mode>,
}

#[derive(Deserialize)]
struct Reject {
    topic: String,
    reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    topic: String,
    chain_id: String,
    request: RpcRequest,
    deadline_s: Option<u64>,
}

#[derive(Deserialize)]
struct Respond {
    topic: String,
    id: i64,
    #[serde(default, with = "embedded_json::option")]
    result: Option<String>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Emit {
    topic: String,
    chain_id: String,
    event: SessionEventData,
}

#[derive(Deserialize)]
struct Update {
    topic: String,
    namespaces: Namespaces,
}

#[derive(Deserialize)]
struct Extend {
    topic: String,
    expiry: Option<u64>,
}

pub async fn call(
    client: &SignClient,
    method: &str,
    params_json: &str,
) -> Result<String, SignError> {
    let p = if params_json.trim().is_empty() {
        Value::Null
    } else {
        ijson::parse(params_json)
            .map(|x| x.value)
            .map_err(|e| SignError::new("invalid_params", e.0))?
    };
    let c = client.clone();
    let out: Value = match method {
        "connect" => {
            let a: Connect = args(&p)?;
            to_json(&boxed(c.connect(a.params, a.intent)).await?)
        }
        "pair" => to_json(&boxed(c.pair(args::<Pair>(&p)?.uri)).await?),
        "approve" => {
            let a: Approve = args(&p)?;
            to_json(&boxed(c.approve(a.topic, a.namespaces, a.own)).await?)
        }
        "reject" => {
            let a: Reject = args(&p)?;
            boxed(c.reject(a.topic, a.reason)).await?;
            Value::Null
        }
        "request" => {
            let a: Request = args(&p)?;
            let r =
                boxed(c.request(a.topic, a.chain_id, a.request, a.deadline_s))
                    .await?;
            ijson::value(r.as_bytes()).unwrap_or(Value::Null)
        }
        "respond" => {
            let a: Respond = args(&p)?;
            boxed(c.respond(a.topic, a.id, a.result, a.error)).await?;
            Value::Null
        }
        "emit" => {
            let a: Emit = args(&p)?;
            boxed(c.emit(a.topic, a.chain_id, a.event)).await?;
            Value::Null
        }
        "ping" => {
            boxed(c.ping(args::<Topic>(&p)?.topic)).await?;
            Value::Null
        }
        "update" => {
            let a: Update = args(&p)?;
            boxed(c.update(a.topic, a.namespaces)).await?;
            Value::Null
        }
        "extend" => {
            let a: Extend = args(&p)?;
            boxed(c.extend(a.topic, a.expiry)).await?;
            Value::Null
        }
        "disconnect" => {
            boxed(c.disconnect(args::<Topic>(&p)?.topic)).await?;
            Value::Null
        }
        "getSessions" => to_json(&boxed(c.sessions()).await),
        "clientId" => Value::String(client.client_id()),
        other => return Err(SignError::new("unknown_method", other)),
    };
    Ok(out.to_string())
}
