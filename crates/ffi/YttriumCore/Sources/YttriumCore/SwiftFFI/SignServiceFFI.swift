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

public enum SignerError: Error {
    case unknown
}

public typealias OnSign = (String) -> Result<String, SignerError>

public final class Signer {
    
    public let signerId: SignerId
    
    private let onSign: OnSign
    
    public init(signerId: SignerId, onSign: @escaping OnSign) {
        self.signerId = signerId
        self.onSign = onSign
    }
    
    public func sign(message: String) -> Result<String, SignerError> {
        onSign(message)
    }
}

public struct SignerId: Hashable, CustomStringConvertible, RawRepresentable {
    
    public var rawValue: String {
        "\(account)-\(chainId)"
    }
    
    public var description: String {
        rawValue
    }
    
    public let account: String
    public let chainId: Int
    
    public init(account: String, chainId: Int) {
        self.account = account
        self.chainId = chainId
    }
    
    public init?(rawValue: String) {
        let idComponents = rawValue.split(separator: "-")
        guard idComponents.count == 2 else {
            return nil
        }
        let account = String(idComponents[0])
        guard let chainId = Int(idComponents[1]) else {
            return nil
        }
        self.account = account
        self.chainId = chainId
    }
}

public final class SignerServiceFFI {
    
    public let signer: Signer
   
    public init(signer_id: RustString) {
        let idString = signer_id.toString()
        let signerId = SignerId(rawValue: idString)!
        self.signer = Signers.shared.signer(id: signerId)!
    }
    
    public func sign(message: RustString) -> FFIStringResult {
        signer.sign(message: message.toString())
            .mapError(\.localizedDescription)
            .ffi
    }
}

extension String: Error {}

extension Result where Success == String, Failure == String {
    
    public var ffi: FFIStringResult {
        switch self {
        case .success(let value):
            return .Ok(value.intoRustString())
        case .failure(let error):
            return .Err(error.localizedDescription.intoRustString())
        }
    }
}
