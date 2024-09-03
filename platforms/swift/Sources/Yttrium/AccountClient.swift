import Foundation
import YttriumCore

public final class AccountClient: AccountClientProtocol {
    
    public var onSign: OnSign? {
        didSet {
            if let onSign = onSign {
                register(onSign: onSign)
            }
        }
    }
    
    public let chainId: Int
    
    private let entryPoint: String
    
    private let coreAccountClient: YttriumCore.FFIAccountClient
    
    public init(ownerAddress: String, entryPoint: String, chainId: Int, onSign: OnSign?) {
        let config: FFIAccountClientConfig = FFIAccountClientConfig(
            owner_address: ownerAddress.intoRustString(),
            chain_id: Int64(chainId),
            config: .init(
                endpoints: .init(
                    rpc: .init(
                        api_key: "".intoRustString(),
                        base_url: "https://localhost:8545".intoRustString() // TODO
                    ),
                    bundler: .init(
                        api_key: "".intoRustString(),
                        base_url: "https://localhost:4337".intoRustString() // TODO
                    ),
                    paymaster: .init(
                        api_key: "".intoRustString(),
                        base_url: "https://localhost:4337".intoRustString() // TODO
                    )
                )
            )
        )
        self.chainId = chainId
        self.entryPoint = entryPoint
        self.coreAccountClient = FFIAccountClient(config)
    }
    
    private func register(onSign: @escaping OnSign) {
        let signer: Signer = .init(
            signerId: .init(
                account: entryPoint,
                chainId: chainId
            ),
            onSign: { message in
                onSign(message)
                    .mapError(YttriumCore.SignerError.from(error:))
            }
        )
        Signers.shared.register(signer: signer)
    }
    
    public func sendTransaction(_ transaction: Transaction) async throws -> String {
        try await coreAccountClient.send_transaction(transaction.ffi).toString()
    }
    
    public func sendBatchTransaction(_ batch: [Transaction]) async throws -> String {
        fatalError("Not yet implemented")
    }
    
    public func getAddress() async throws -> String {
        try await coreAccountClient.get_address().toString()
    }
    
    public func signMessage(_ message: String) throws -> String {
        fatalError("Not yet implemented")
    }
}

extension Transaction {
    
    var ffi: FFITransaction {
        FFITransaction(
            to: to,
            value: value,
            data: data
        )
    }
}

extension YttriumCore.SignerError {
    static func from(error: SignerError) -> Self {
        switch error {
        case .unknown:
            return .unknown
        }
    }
}
