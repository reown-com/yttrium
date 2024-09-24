import Foundation
import YttriumCore

public final class AccountClient: AccountClientProtocol {
    
    public let ownerAddress: String
    
    public let chainId: Int
    
    private let entryPoint: String
    
    private let coreAccountClient: YttriumCore.FFIAccountClient
    
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
        self.coreAccountClient = FFIAccountClient(ffiConfig)
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
        self.coreAccountClient = FFIAccountClient(ffiConfig)
        
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
    
    func signMessageWithMnemonic(
        message: String,
        mnemonic: String
    ) throws -> String {
        try coreAccountClient.sign_message_with_mnemonic(
            message.intoRustString(),
            mnemonic.intoRustString()
        )
        .toString()
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
