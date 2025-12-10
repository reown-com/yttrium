use {
    crate::sign::protocol_types::Relay, relay_rpc::domain::Topic,
    std::collections::HashMap, url::Url, x25519_dalek::StaticSecret,
};

#[derive(Debug, thiserror::Error)]
#[error("Pairing URI parse error: {0}")]
pub enum Error {
    #[error("Invalid URI")]
    InvalidUri,

    #[error("Invalid scheme")]
    InvalidScheme,

    #[error("URI has host")]
    UriHasHost,

    #[error("Missing topic")]
    MissingTopic,

    #[error("Missing version")]
    MissingVersion,

    #[error("Invalid version")]
    InvalidVersion,

    #[error("Topic wrong length")]
    TopicWrongLength,

    #[error("Topic invalid hex")]
    TopicInvalidHex,

    #[error("Missing relay protocol")]
    MissingRelayProtocol,

    #[error("Invalid relay protocol")]
    InvalidRelayProtocol,

    #[error("Missing sym key")]
    MissingSymKey,

    #[error("Sym key wrong length")]
    SymKeyWrongLength,

    #[error("Sym key invalid hex")]
    SymKeyInvalidHex,

    #[error("Invalid expiry timestamp")]
    InvalidExpiryTimestamp,
}

pub struct PairingUri {
    pub topic: Topic,
    pub sym_key: [u8; 32],
    #[allow(unused)]
    pub expiry_timestamp: Option<u64>,
}

pub fn parse(uri: &str) -> Result<PairingUri, Error> {
    let uri = Url::parse(uri).map_err(|_| Error::InvalidUri)?;
    if uri.scheme() != "wc" {
        return Err(Error::InvalidScheme);
    }
    if uri.has_host() {
        return Err(Error::UriHasHost);
    }
    let topic = {
        let path = uri.path();
        let mut path_parts = path.split("@");
        let topic = path_parts.next().ok_or(Error::MissingTopic)?;
        let version = path_parts.next().ok_or(Error::MissingVersion)?;
        if version != "2" {
            return Err(Error::InvalidVersion);
        }
        if topic.len() != 64 {
            return Err(Error::TopicWrongLength);
        }
        if hex::decode(topic).is_err() {
            return Err(Error::TopicInvalidHex);
        }
        Topic::new(topic.to_string().into())
    };
    let query = uri.query_pairs().collect::<HashMap<_, _>>();

    let relay_protocol =
        query.get("relay-protocol").ok_or(Error::MissingRelayProtocol)?;
    if relay_protocol != "irn" {
        return Err(Error::InvalidRelayProtocol);
    }

    let sym_key = hex::decode(
        query.get("symKey").ok_or(Error::MissingSymKey)?.as_bytes(),
    )
    .map_err(|_| Error::SymKeyInvalidHex)?
    .try_into()
    .map_err(|_| Error::SymKeyWrongLength)?;

    let expiry_timestamp = query
        .get("expiryTimestamp")
        .map(|s| s.parse::<u64>().map_err(|_| Error::InvalidExpiryTimestamp))
        .transpose()?;

    Ok(PairingUri { topic, sym_key, expiry_timestamp })
}

pub fn format(
    topic: &Topic,
    sym_key: &StaticSecret,
    relay: &Relay,
    expiry: u64,
) -> String {
    let protocol = &relay.protocol;
    let sym_key = hex::encode(sym_key.as_bytes());
    format!(
        "wc:{topic}@2?relay-protocol={protocol}&symKey={sym_key}&expiryTimestamp={expiry}",
    )
}

#[cfg(test)]
mod tests {
    use {super::*, crate::sign::utils::topic_from_sym_key};

    #[test]
    fn test_parse() {
        let uri = "wc:0c814f7d2d56c0e840f75612addaa170af479b1c8499632430b41c298bf49907@2?relay-protocol=irn&symKey=d69745274f07e8619671a527943b38a11dce540be5c0965f04cdece9912bdfd5&expiryTimestamp=1752843899";
        let result = parse(uri).unwrap();
        assert_eq!(
            result.topic,
            Topic::new("0c814f7d2d56c0e840f75612addaa170af479b1c8499632430b41c298bf49907".into())
        );
        assert_eq!(
            &result.sym_key,
            &hex::decode("d69745274f07e8619671a527943b38a11dce540be5c0965f04cdece9912bdfd5")
                .unwrap()[..]
        );
        assert_eq!(result.expiry_timestamp, Some(1752843899));
    }

    #[test]
    fn test_parse_invalid_uri() {
        let uri = "";
        let result = parse(uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_and_parse_compat() {
        let sym_key = StaticSecret::from([0u8; 32]);
        let topic = topic_from_sym_key(sym_key.as_bytes());
        let relay = Relay { protocol: "irn".to_string() };
        let expiry = 1752843899;
        let uri = format(&topic, &sym_key, &relay, expiry);

        assert_eq!(
            uri,
            "wc:66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925@2?relay-protocol=irn&symKey=0000000000000000000000000000000000000000000000000000000000000000&expiryTimestamp=1752843899"
        );

        assert!(uri.starts_with("wc:"));
        assert!(uri.contains("@2"));
        assert!(uri.contains("?relay-protocol=irn"));
        assert!(uri.contains("&symKey="));
        assert!(uri.contains("&expiryTimestamp="));

        let result = parse(&uri).unwrap();
        assert_eq!(result.topic, topic);
        assert_eq!(&result.sym_key, sym_key.as_bytes());
        assert_eq!(result.expiry_timestamp, Some(expiry));
    }
}
