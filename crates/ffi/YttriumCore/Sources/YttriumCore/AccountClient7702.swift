import Foundation

public final class AccountClient7702 {
    
    public let ownerAddress: String
    
    public let chainId: Int
    
    private let entryPoint: String
    
    private let core7702AccountClient: YttriumCore.FFI7702AccountClient
    
    public convenience init(
        ownerAddress: String,
        entryPoint: String,
        chainId: Int,
        config: Config,
        safe: Bool
    ) {
        self.init(
            ownerAddress: ownerAddress,
            entryPoint: entryPoint,
            chainId: chainId,
            config: config,
            signerType: .privateKey,
            safe: safe
        )
    }
    
    init(
        ownerAddress: String,
        entryPoint: String,
        chainId: Int,
        config: Config,
        signerType: SignerType,
        safe: Bool
    ) {
        let ffiConfig: FFIAccountClientConfig = FFIAccountClientConfig(
            owner_address: ownerAddress.intoRustString(),
            chain_id: UInt64(chainId),
            config: config.ffi,
            signer_type: signerType.toRustString(),
            safe: safe
        )
        self.ownerAddress = ownerAddress
        self.chainId = chainId
        self.entryPoint = entryPoint
        self.core7702AccountClient = FFI7702AccountClient(ffiConfig)
    }
    
    init(
        ownerAddress: String,
        entryPoint: String,
        chainId: Int,
        config: Config,
        signer: Signer,
        safe: Bool
    ) {
        let ffiConfig: FFIAccountClientConfig = FFIAccountClientConfig(
            owner_address: ownerAddress.intoRustString(),
            chain_id: UInt64(chainId),
            config: config.ffi,
            signer_type: signer.signerType.toRustString(),
            safe: safe
        )
        self.ownerAddress = ownerAddress
        self.chainId = chainId
        self.entryPoint = entryPoint
        self.core7702AccountClient = FFI7702AccountClient(ffiConfig)
        
        register(signer: signer)
    }
    
    public func register(privateKey: String) {
        let signerId: SignerId = .init(
            signerType: .privateKey,
            account: ownerAddress,
            chainId: chainId
        )
        let privateKeySigner = PrivateKeySigner(
            id: signerId,
            privateKey: privateKey
        )
        register(signer: .privateKey(privateKeySigner))
    }
    
    func register(onSign: @escaping OnSign) {
        let signerId: SignerId = .init(
            signerType: .native,
            account: ownerAddress,
            chainId: chainId
        )
        let nativeSigner = NativeSigner(
            id: signerId,
            onSign: { message in
                onSign(message)
            }
        )
        register(signer: .native(nativeSigner))
    }
    
    private func register(signer: Signer) {
        Signers.shared.register(signer: signer)
    }
    
    public func sendBatchTransaction(_ batch: [Transaction]) async throws -> String {
        let ffiBatch = batch.map(\.ffi)
        let batchJSONData = try JSONEncoder().encode(ffiBatch)
        let batchJSONString = String(decoding: batchJSONData, as: UTF8.self)
        return try await core7702AccountClient.send_batch_transaction(batchJSONString).toString()
    }
}
