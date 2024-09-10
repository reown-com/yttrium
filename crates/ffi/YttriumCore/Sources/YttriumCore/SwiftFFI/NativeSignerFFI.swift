import Foundation

public final class Signers {
    
    public static var shared = Signers()
    
    private var signers: [SignerId: Signer]
    
    public init(signers: [Signer] = []) {
        let signerKeyValues = signers.map { ($0.signerId, $0) }
        self.signers = Dictionary(uniqueKeysWithValues: signerKeyValues)
    }
    
    public func signer(id: SignerId) -> Signer? {
        self.signers[id]
    }
    
    public func register(signer: Signer) {
        signers[signer.signerId] = signer
    }
    
    public func register(signers: [Signer]) {
        for signer in signers {
            register(signer: signer)
        }
    }
}

public typealias OnSign = (String) -> Result<String, SignerError>

public final class NativeSigner: Identifiable {
    
    public let id: SignerId
    
    private let onSign: OnSign
    
    public init(id: SignerId, onSign: @escaping OnSign) {
        self.id = id
        self.onSign = onSign
    }
    
    public func sign(message: String) -> Result<String, SignerError> {
        onSign(message)
    }
}

public struct SignerId: Hashable, CustomStringConvertible, RawRepresentable {
    
    public var rawValue: String {
        "\(signerType)-\(account)-\(chainId)"
    }
    
    public var description: String {
        rawValue
    }
    
    public let signerType: SignerType
    public let account: String
    public let chainId: Int
    
    public init(signerType: SignerType, account: String, chainId: Int) {
        self.signerType = signerType
        self.account = account
        self.chainId = chainId
    }
    
    public init?(rawValue: String) {
        let idComponents = rawValue.split(separator: "-")
        guard idComponents.count == 3 else {
            return nil
        }
        guard let signerType = SignerType(rawValue: String(idComponents[0])) else {
            return nil
        }
        let account = String(idComponents[1])
        guard let chainId = Int(idComponents[2]) else {
            return nil
        }
        self.signerType = signerType
        self.account = account
        self.chainId = chainId
    }
}

public final class NativeSignerFFI {
    
    public let signer: NativeSigner
   
    public init(signer_id: RustString) {
        let idString = signer_id.toString()
        let signerId = SignerId(rawValue: idString)!
        self.signer = Signers.shared.signer(id: signerId)!.nativeSigner!
    }
    
    public func sign(message: RustString) -> FFIStringResult {
        signer.sign(message: message.toString())
            .mapError(\.localizedDescription)
            .ffi
    }
}
