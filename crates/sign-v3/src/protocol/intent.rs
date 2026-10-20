use super::types::{DappIntent, Mode, PeerRequirement, WalletIntent};

pub fn satisfies(mode: Mode, req: PeerRequirement) -> bool {
    match req {
        PeerRequirement::None => true,
        PeerRequirement::Monitored => mode == Mode::Monitored,
        PeerRequirement::Sealed => mode == Mode::Sealed,
    }
}

/// The wallet's own direction must meet the dApp's requirement, and a sealed direction
/// counts only with the dApp project's private-channel entitlement.
pub fn bind_compatibility(
    dapp: &DappIntent,
    wallet: &WalletIntent,
    entitled: bool,
) -> Option<&'static str> {
    if !satisfies(wallet.own, dapp.peer) {
        return Some("incompatible_intent");
    }
    if wallet.own == Mode::Sealed && !entitled {
        return Some("not_entitled");
    }
    None
}
