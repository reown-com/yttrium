use alloy::sol;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Token {
    Usdc,
}

sol! {
    #[sol(rpc)]
    contract ERC20 {
        function transfer(address to, uint256 amount);
        function approve(address spender, uint256 amount) public returns (bool);
        function balanceOf(address _owner) public view returns (uint256 balance);
    }
}
