
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
}
