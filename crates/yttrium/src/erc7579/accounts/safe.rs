use alloy::primitives::{Address, aliases::U192};

// encodeValidatorNonce in Rhinestone SDK
pub fn encode_validator_key(validator_module_address: Address) -> U192 {
    U192::from_be_bytes({
        let mut key = [0u8; 24];
        key[..20].copy_from_slice(&validator_module_address.into_array());
        key
    })
}

#[cfg(test)]
mod test {
    use {
        super::*,
        alloy::primitives::{address, aliases::B192, bytes, fixed_bytes},
    };

    #[test]
    fn test_encode_validator_key() {
        assert_eq!(
            B192::from(
                encode_validator_key(address!(
                    "00112233445566778899aabbccddeeff00112233"
                ))
                .to_be_bytes()
            ),
            fixed_bytes!("00112233445566778899aabbccddeeff0011223300000000")
        );
    }

    #[test]
    fn test_key_from_validator_address() {
        let validator_address =
            address!("abababababababababababababababababababab");
        let key = encode_validator_key(validator_address);
        assert_eq!(
            key.to_be_bytes_vec(),
            bytes!("abababababababababababababababababababab00000000").to_vec()
        );
    }
}
