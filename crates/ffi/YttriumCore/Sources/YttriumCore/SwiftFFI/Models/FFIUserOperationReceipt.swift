import Foundation

public struct UserOperationReceipt: Codable {
    
    public struct Receipt: Codable {
        public let transactionHash: String
        public let transactionIndex: String
        public let block_hash: String
        public let block_number: String
        public let from: String
        public let to: String
        public let cumulativeGasUsed: String
        public let gas_used: String
        public let contractAddress: String?
        public let status: String
        public let logsBloom: String
        public let effectiveGasPrice: String
        
        public init(
            transactionHash: String,
            transactionIndex: String,
            block_hash: String,
            block_number: String,
            from: String,
            to: String,
            cumulativeGasUsed: String,
            gas_used: String,
            contractAddress: String?,
            status: String,
            logsBloom: String,
            effectiveGasPrice: String
        ) {
            self.transactionHash = transactionHash
            self.transactionIndex = transactionIndex
            self.block_hash = block_hash
            self.block_number = block_number
            self.from = from
            self.to = to
            self.cumulativeGasUsed = cumulativeGasUsed
            self.gas_used = gas_used
            self.contractAddress = contractAddress
            self.status = status
            self.logsBloom = logsBloom
            self.effectiveGasPrice = effectiveGasPrice
        }
    }
    
    public let userOpHash: String
    public let entryPoint: String
    public let sender: String
    public let nonce: String
    public let paymaster: String
    public let actualGasCost: String
    public let actualGasUsed: String
    public let success: Bool
    public let receipt: Receipt
    
    public init(
        userOpHash: String,
        entryPoint: String,
        sender: String,
        nonce: String,
        paymaster: String,
        actualGasCost: String,
        actualGasUsed: String,
        success: Bool,
        receipt: Receipt
    ) {
        self.userOpHash = userOpHash
        self.entryPoint = entryPoint
        self.sender = sender
        self.nonce = nonce
        self.paymaster = paymaster
        self.actualGasCost = actualGasCost
        self.actualGasUsed = actualGasUsed
        self.success = success
        self.receipt = receipt
    }
}
