use alloy::sol;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    SafeProxyFactory,
    "../../target/.foundry/forge/out/SafeProxyFactory.sol/SafeProxyFactory.json"
    // concat!(env!("OUT_DIR"), "/../../../../.foundry/forge/out/SafeProxyFactory.sol/SafeProxyFactory.json")
);

// const SUCCESS_RESULT: u8 = 0x01;
// sol! {
//   contract ValidateSigOffchain {
//     constructor (address _signer, bytes32 _hash, bytes memory _signature);
//   }
// }
// const VALIDATE_SIG_OFFCHAIN_BYTECODE: &[u8] = include_bytes!(concat!(
//     env!("OUT_DIR"),
//     "/../../../../.foundry/forge/out/Erc6492.sol/ValidateSigOffchain.bytecode"
// ));
