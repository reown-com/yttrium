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


public protocol AccountClientProtocol {
    
    var chainId: Int { get }
    
    init(ownerAddress: String, entryPoint: String, chainId: Int, config: Config, safe: Bool)
    
    func register(privateKey: String)

    func sendTransactions(_ transactions: [Transaction]) async throws -> String
    func getAddress() async throws -> String
    func signMessage(_ message: String) throws -> String
}
