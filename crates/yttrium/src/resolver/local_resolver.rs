use crate::descriptors::aave::{AAVE_LPV2, AAVE_LPV3, AAVE_WETH_GATEWAY_V3};

pub trait ResolverClient {
    fn resolve_descriptor(&self, caip10: &str) -> Option<&'static str>;
}

#[derive(Debug, Default)]
pub struct LocalResolver;

impl LocalResolver {
    pub fn new() -> Self {
        Self
    }
}

impl ResolverClient for LocalResolver {
    fn resolve_descriptor(&self, caip10: &str) -> Option<&'static str> {
        let (chain_id, address) = parse_caip10(caip10)?;
        match (chain_id, address.as_str()) {
            (1, "0x7d2768de32b0b80b7a3454c06bdac94a69ddc7a9") => {
                Some(AAVE_LPV2)
            }
            (137, "0x8dff5e27ea6b7ac08ebfdf9eb090f32ee9a30fcf") => {
                Some(AAVE_LPV2)
            }
            (43114, "0x4f01aed16d97e3ab5ab2b501154dc9bb0f1a5a2c") => {
                Some(AAVE_LPV2)
            }
            (1, "0xd01607c3c5ecaba394d8be377a08590149325722") => {
                Some(AAVE_WETH_GATEWAY_V3)
            }
            (10, "0x5f2508cae9923b02316254026cd43d7902866725") => {
                Some(AAVE_WETH_GATEWAY_V3)
            }
            (100, "0x721b9abab6511b46b9ee83a1aba23bdacb004149") => {
                Some(AAVE_WETH_GATEWAY_V3)
            }
            (137, "0xbc302053db3aa514a3c86b9221082f162b91ad63") => {
                Some(AAVE_WETH_GATEWAY_V3)
            }
            (146, "0x061d8e131f26512348ee5fa42e2df1ba9d6505e9") => {
                Some(AAVE_WETH_GATEWAY_V3)
            }
            (324, "0xae2b00d676130bdf22582781bbba8f4f21e8b0ff") => {
                Some(AAVE_WETH_GATEWAY_V3)
            }
            (1868, "0x6376d4df995f32f308f2d5049a7a320943023232") => {
                Some(AAVE_WETH_GATEWAY_V3)
            }
            (8453, "0xa0d9c1e9e48ca30c8d8c3b5d69ff5dc1f6dffc24") => {
                Some(AAVE_WETH_GATEWAY_V3)
            }
            (9745, "0x54bdcc37c4143f944a3ee51c892a6cbdf305e7a0") => {
                Some(AAVE_WETH_GATEWAY_V3)
            }
            (42161, "0x5283beced7adf6d003225c13896e536f2d4264ff") => {
                Some(AAVE_WETH_GATEWAY_V3)
            }
            (43114, "0x2825ce5921538d17cc15ae00a8b24ff759c6cdae") => {
                Some(AAVE_WETH_GATEWAY_V3)
            }
            (59144, "0x31a239f3e39c5d8ba6b201ba81ed584492ae960f") => {
                Some(AAVE_WETH_GATEWAY_V3)
            }
            (534352, "0xe79ca44408dae5a57e2a9594532f1e84d2edaa4") => {
                Some(AAVE_WETH_GATEWAY_V3)
            }
            (1, "0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2") => {
                Some(AAVE_LPV3)
            }
            (8453, "0xa238dd80c259a72e81d7e4664a9801593f98d1c5") => {
                Some(AAVE_LPV3)
            }
            (42220, "0x3e59a31363e2ad014dcbc521c4a0d5757d9f3402") => {
                Some(AAVE_LPV3)
            }
            (59144, "0xc47b8c00b0f69a36fa203ffea0334874574a8ac") => {
                Some(AAVE_LPV3)
            }
            (1088, "0x90df02551bb792286e8d4f13e0e357b4bf1d6a57") => {
                Some(AAVE_LPV3)
            }
            (146, "0x5362dbb1e601abf3a4c14c22ffeda64042e5eaa3") => {
                Some(AAVE_LPV3)
            }
            (100, "0xb50201558b00496a145fe76f7424749556e326d8") => {
                Some(AAVE_LPV3)
            }
            (534352, "0x11fcfe756c05ad438e312a7fd934381537d3cffe") => {
                Some(AAVE_LPV3)
            }
            (324, "0x78e30497a3c7527d953c6b1e3541b021a98ac43c") => {
                Some(AAVE_LPV3)
            }
            (137, "0x794a61358d6845594f94dc1db02a252b5b4814ad") => {
                Some(AAVE_LPV3)
            }
            (1868, "0xdd3d7a7d03d9fd9ef45f3e587287922ef65ca38b") => {
                Some(AAVE_LPV3)
            }
            (42161, "0x794a61358d6845594f94dc1db02a252b5b4814ad") => {
                Some(AAVE_LPV3)
            }
            (10, "0x794a61358d6845594f94dc1db02a252b5b4814ad") => {
                Some(AAVE_LPV3)
            }
            (43114, "0x794a61358d6845594f94dc1db02a252b5b4814ad") => {
                Some(AAVE_LPV3)
            }
            (9745, "0x925a2a7214ed92428b5b1b090f80b25700095e12") => {
                Some(AAVE_LPV3)
            }
            _ => None,
        }
    }
}

fn parse_caip10(caip10: &str) -> Option<(u64, String)> {
    let mut parts = caip10.split(':');
    let namespace = parts.next()?;
    let reference = parts.next()?;
    let address = parts.next()?;

    if namespace != "eip155" || parts.next().is_some() {
        return None;
    }

    let chain_id = reference.parse().ok()?;
    let normalized_address = address.trim().to_ascii_lowercase();

    if !normalized_address.starts_with("0x") {
        return None;
    }

    Some((chain_id, normalized_address))
}
