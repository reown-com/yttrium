import Foundation

public struct UserOperationReceipt: Codable {

    public struct Receipt: Codable {
        public let transactionHash: String
        public let transactionIndex: String
        public let blockHash: String
        public let blockNumber: String
        public let from: String
        public let to: String
        public let cumulativeGasUsed: String
        public let gasUsed: String
        public let contractAddress: String?
        public let status: String
        public let logsBloom: String
        public let effectiveGasPrice: String

        public init(
            transactionHash: String,
            transactionIndex: String,
            blockHash: String,
            blockNumber: String,
            from: String,
            to: String,
            cumulativeGasUsed: String,
            gasUsed: String,
            contractAddress: String?,
            status: String,
            logsBloom: String,
            effectiveGasPrice: String
        ) {
            self.transactionHash = transactionHash
            self.transactionIndex = transactionIndex
            self.blockHash = blockHash
            self.blockNumber = blockNumber
            self.from = from
            self.to = to
            self.cumulativeGasUsed = cumulativeGasUsed
            self.gasUsed = gasUsed
            self.contractAddress = contractAddress
            self.status = status
            self.logsBloom = logsBloom
            self.effectiveGasPrice = effectiveGasPrice
        }
    }

    public struct Log: Codable {
        public let address: String
        public let topics: [String]
        public let data: String
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
    public let logs: [Log]

    public init(
        userOpHash: String,
        entryPoint: String,
        sender: String,
        nonce: String,
        paymaster: String,
        actualGasCost: String,
        actualGasUsed: String,
        success: Bool,
        receipt: Receipt,
        logs: [Log]
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
        self.logs = logs
    }
}
