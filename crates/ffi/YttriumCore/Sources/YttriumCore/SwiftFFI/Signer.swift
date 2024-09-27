import Foundation

public enum Signer {
    case native(NativeSigner)
    case privateKey(PrivateKeySigner)
    
    public var signerType: SignerType {
        switch self {
        case .native:
            return .native
        case .privateKey:
            return .privateKey
        }
    }
    
    public var signerId: SignerId {
        switch self {
        case .native(let native):
            return native.id
        case .privateKey(let privateKey):
            return privateKey.id
        }
    }
    
    public var privateKeySigner: PrivateKeySigner? {
        switch self {
        case .native:
            return nil
        case .privateKey(let privateKey):
            return privateKey
        }
    }
    
    public var nativeSigner: NativeSigner? {
        switch self {
        case .native(let native):
            return native
        case .privateKey:
            return nil
        }
    }
}

public enum SignerType: String, Codable {
    case native = "Native"
    case privateKey = "PrivateKey"
    
    public func toRustString() -> RustString {
        rawValue.intoRustString()
    }
}

public enum SignerError: Error {
    case unknown
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
