import Foundation
import YttriumCore

public final class AccountClient7702 {
    
    public var onSign: OnSign? {
        didSet {
            if let onSign = onSign {
                register(onSign: onSign)
            }
        }
    }
    
    public let chainId: Int
    
    private let entryPoint: String
    
    private let core7702AccountClient: YttriumCore.FFI7702AccountClient
    
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
                    )
                )
            )
        )
        self.chainId = chainId
        self.entryPoint = entryPoint
        self.core7702AccountClient = FFI7702AccountClient(config)
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
    
    public func sendBatchTransaction(_ batch: [Transaction]) async throws -> String {
        let ffiBatch = batch.map(\.ffi)
        let batchJSONData = try JSONEncoder().encode(ffiBatch)
        let batchJSONString = String(decoding: batchJSONData, as: UTF8.self)
        return try await core7702AccountClient.send_batch_transaction(batchJSONString).toString()
    }
}
