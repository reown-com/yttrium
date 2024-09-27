import Foundation

public final class PrivateKeySignerFFI {
    
    public let signerId: SignerId
    
    private let pK: String
   
    public init(signer_id: RustString) {
        let idString = signer_id.toString()
        let signerId = SignerId(rawValue: idString)!
        self.signerId = signerId
        self.pK = Signers.shared.signer(id: signerId)!.privateKeySigner!.privateKey
    }
    
    public func private_key() -> FFIStringResult {
        .Ok(pK.intoRustString())
    }
}

public struct PrivateKeySigner: Identifiable {
    
    public let id: SignerId
    
    public let privateKey: String
    
    public init(id: SignerId, privateKey: String) {
        self.id = id
        self.privateKey = privateKey
    }
}
