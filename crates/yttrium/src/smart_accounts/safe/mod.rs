use alloy::sol;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    SafeProxyFactory,
    "../../target/.foundry/forge/out/SafeProxyFactory.sol/SafeProxyFactory.json"
    // concat!(env!("OUT_DIR"), "/../../../../.foundry/forge/out/SafeProxyFactory.sol/SafeProxyFactory.json")
);
