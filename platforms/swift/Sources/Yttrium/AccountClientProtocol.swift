import Foundation

public struct Transaction: Codable, Equatable {
    let to: String
    let value: String
    let data: String

    public init(to: String, value: String, data: String) {
        self.to = to
        self.value = value
        self.data = data
    }
}

public enum SignerError: Error {
    case unknown
}

public typealias OnSign = (String) -> Result<String, SignerError>

public protocol AccountClientProtocol {
    
    var onSign: OnSign? { get set }
    
    var chainId: Int { get }
    
    init(ownerAddress: String, entryPoint: String, chainId: Int, onSign: OnSign?)

    func sendTransaction(_ transaction: Transaction) async throws -> String
    func sendBatchTransaction(_ batch: [Transaction]) async throws -> String
    func getAddress() async throws -> String
    func signMessage(_ message: String) throws -> String
}
