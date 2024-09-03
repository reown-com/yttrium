pragma solidity ^0.8.20;

contract Account7702 {
    constructor() {} // TODO need owner

    struct Call {
        bytes data;
        address to;
        uint256 value;
        bytes signature; // TODO proper type
    }

    function execute(Call[] calldata calls) external payable {
        // TODO how to authenticate signture
        for (uint256 i = 0; i < calls.length; i++) {
            Call memory call = calls[i];
            (bool success, ) = call.to.call{value: call.value}(call.data);
            require(success, "call reverted");
        }
    }
}
