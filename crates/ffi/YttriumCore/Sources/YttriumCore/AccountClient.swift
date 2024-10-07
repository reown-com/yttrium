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
    
    public func sendTransactions(_ transactions: [Transaction]) async throws -> String {
        let jsonEncoder = JSONEncoder()

        let jsonStrings = try transactions.map { transaction in
            let jsonData = try jsonEncoder.encode(transaction)
            guard let jsonString = String(data: jsonData, encoding: .utf8) else {
                throw NSError(domain: "EncodingError", code: -1, userInfo: [NSLocalizedDescriptionKey: "Failed to convert JSON data to string"])
            }
            return jsonString
        }

        let rustVec = createRustVec(from: jsonStrings)

        return try await coreAccountClient.send_transactions(rustVec).toString()
    }
        
    private func createRustVec(from strings: [String]) -> RustVec<RustString> {
        let rustVec = RustVec<RustString>()

        for string in strings {
            let rustString = RustString(string)
            rustVec.push(value: rustString)
        }

        return rustVec
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

    public func waitForUserOperationReceipt(
        userOperationHash: String
    ) async throws -> UserOperationReceipt {
        let jsonString = try await coreAccountClient
            .wait_for_user_operation_receipt(
                userOperationHash.intoRustString()
            )
            .toString()
        let jsonData = Data(jsonString.utf8)
        let receipt = try JSONDecoder().decode(
            UserOperationReceipt.self,
            from: jsonData
        )
        return receipt
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
