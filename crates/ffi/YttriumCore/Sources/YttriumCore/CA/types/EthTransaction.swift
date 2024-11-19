
import Foundation

public struct EthTransaction: Codable {
    public let from: String
    public let to: String
    public let value: String
    public let gas: String
    public let gasPrice: String
    public let data: String
    public let nonce: String
    public let maxFeePerGas: String
    public let maxPriorityFeePerGas: String
    public let chainId: String

    public init(
        from: String,
        to: String,
        value: String,
        gas: String,
        gasPrice: String,
        data: String,
        nonce: String, maxFeePerGas: String,
        maxPriorityFeePerGas: String,
        chainId: String
    ) {
        self.from = from
        self.to = to
        self.value = value
        self.gas = gas
        self.gasPrice = gasPrice
        self.data = data
        self.nonce = nonce
        self.maxFeePerGas = maxFeePerGas
        self.maxPriorityFeePerGas = maxPriorityFeePerGas
        self.chainId = chainId
    }
}
