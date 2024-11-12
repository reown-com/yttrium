import Foundation

public struct OwnerSignature: Codable {
    public var owner: String
    public var signature: String
    public init(owner: String, signature: String) {
        self.owner = owner
        self.signature = signature
    }
}

public struct PreparedSendTransaction: Codable {
    public var hash: String
    public var doSendTransactionParams: String
}

public struct PreparedSignMessage: Codable {
    let hash: String
}

public final class AccountClient: AccountClientProtocol {
    struct Errors: LocalizedError {
        let message: String

        var errorDescription: String? {
            return message
        }
    }

    public let ownerAddress: String
    
    public let chainId: Int
    
    private let entryPoint: String
    
    private let coreAccountClient: FFIAccountClient

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

    public func prepareSendTransactions(_ transactions: [Transaction]) async throws -> PreparedSendTransaction {
        let jsonEncoder = JSONEncoder()

        let jsonStrings = try transactions.map { transaction in
            let jsonData = try jsonEncoder.encode(transaction)
            guard let jsonString = String(data: jsonData, encoding: .utf8) else {
                throw NSError(domain: "EncodingError", code: -1, userInfo: [NSLocalizedDescriptionKey: "Failed to convert JSON data to string"])
            }
            return jsonString
        }

        let rustVec = createRustVec(from: jsonStrings)

        let ffiPreparedSendTransaction =  try await coreAccountClient.prepare_send_transactions(rustVec)
        return PreparedSendTransaction(
            hash: ffiPreparedSendTransaction.hash.toString(),
            doSendTransactionParams: ffiPreparedSendTransaction.do_send_transaction_params.toString()
        )
    }

    public func doSendTransaction(signatures: [OwnerSignature], params: String) async throws -> String {
        let jsonEncoder = JSONEncoder()

        let jsonSignatures = try signatures.map { signature in
            let jsonData = try jsonEncoder.encode(signature)
            guard let jsonString = String(data: jsonData, encoding: .utf8) else {
                throw NSError(domain: "EncodingError", code: -1, userInfo: [NSLocalizedDescriptionKey: "Failed to convert signature JSON data to string"])
            }
            return jsonString
        }

        let rustSignatures = createRustVec(from: jsonSignatures)

        do {
            return try await coreAccountClient.do_send_transaction(rustSignatures, RustString(params)).toString()
        } catch let ffiError as FFIError {
            switch ffiError {
            case .Unknown(let x):
                let errorMessage = x.toString()
                throw Errors(message: errorMessage)
            }
        } catch {
            throw error
        }
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

    public func prepareSignMessage(_ messageHash: String) async throws -> PreparedSignMessage {
        let messageHash = messageHash.intoRustString()
        let ffiPrepareSignMessage = try await coreAccountClient.prepare_sign_message(messageHash)
        return PreparedSignMessage(hash: ffiPrepareSignMessage.hash.toString())
    }

    public func doSignMessage(_ signatures: [String]) async throws -> String {
        let rustSignatures = createRustVec(from: signatures)
        return try await coreAccountClient.do_sign_message(rustSignatures).toString()
    }

    public func finalizeSignMessage(_ signatures: [String], signStep3Params: String) async throws -> String {
        let rustSignatures = createRustVec(from: signatures)
        return try await coreAccountClient.finalize_sign_message(rustSignatures, signStep3Params.intoRustString()).toString()
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
