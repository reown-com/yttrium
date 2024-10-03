import RustXcframework
@_cdecl("__swift_bridge__$NativeSignerFFI$new")
func __swift_bridge__NativeSignerFFI_new (_ signer_id: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(NativeSignerFFI(signer_id: RustString(ptr: signer_id))).toOpaque()
}

@_cdecl("__swift_bridge__$NativeSignerFFI$sign")
func __swift_bridge__NativeSignerFFI_sign (_ this: UnsafeMutableRawPointer, _ message: UnsafeMutableRawPointer) -> __swift_bridge__$FFIStringResult {
    Unmanaged<NativeSignerFFI>.fromOpaque(this).takeUnretainedValue().sign(message: RustString(ptr: message)).intoFfiRepr()
}

@_cdecl("__swift_bridge__$PrivateKeySignerFFI$new")
func __swift_bridge__PrivateKeySignerFFI_new (_ signer_id: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(PrivateKeySignerFFI(signer_id: RustString(ptr: signer_id))).toOpaque()
}

@_cdecl("__swift_bridge__$PrivateKeySignerFFI$private_key")
func __swift_bridge__PrivateKeySignerFFI_private_key (_ this: UnsafeMutableRawPointer) -> __swift_bridge__$FFIStringResult {
    Unmanaged<PrivateKeySignerFFI>.fromOpaque(this).takeUnretainedValue().private_key().intoFfiRepr()
}

public struct FFITransaction {
    public var _to: RustString
    public var _value: RustString
    public var _data: RustString

    public init(_to: RustString,_value: RustString,_data: RustString) {
        self._to = _to
        self._value = _value
        self._data = _data
    }

    @inline(__always)
    func intoFfiRepr() -> __swift_bridge__$FFITransaction {
        { let val = self; return __swift_bridge__$FFITransaction(_to: { let rustString = val._to.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), _value: { let rustString = val._value.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), _data: { let rustString = val._data.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); }()
    }
}
extension __swift_bridge__$FFITransaction {
    @inline(__always)
    func intoSwiftRepr() -> FFITransaction {
        { let val = self; return FFITransaction(_to: RustString(ptr: val._to), _value: RustString(ptr: val._value), _data: RustString(ptr: val._data)); }()
    }
}
extension __swift_bridge__$Option$FFITransaction {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFITransaction> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }

    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFITransaction>) -> __swift_bridge__$Option$FFITransaction {
        if let v = val {
            return __swift_bridge__$Option$FFITransaction(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFITransaction(is_some: false, val: __swift_bridge__$FFITransaction())
        }
    }
}
public struct FFIEndpoint {
    public var api_key: RustString
    public var base_url: RustString

    public init(api_key: RustString,base_url: RustString) {
        self.api_key = api_key
        self.base_url = base_url
    }

    @inline(__always)
    func intoFfiRepr() -> __swift_bridge__$FFIEndpoint {
        { let val = self; return __swift_bridge__$FFIEndpoint(api_key: { let rustString = val.api_key.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), base_url: { let rustString = val.base_url.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); }()
    }
}
extension __swift_bridge__$FFIEndpoint {
    @inline(__always)
    func intoSwiftRepr() -> FFIEndpoint {
        { let val = self; return FFIEndpoint(api_key: RustString(ptr: val.api_key), base_url: RustString(ptr: val.base_url)); }()
    }
}
extension __swift_bridge__$Option$FFIEndpoint {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIEndpoint> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }

    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIEndpoint>) -> __swift_bridge__$Option$FFIEndpoint {
        if let v = val {
            return __swift_bridge__$Option$FFIEndpoint(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIEndpoint(is_some: false, val: __swift_bridge__$FFIEndpoint())
        }
    }
}
public struct FFIEndpoints {
    public var rpc: FFIEndpoint
    public var bundler: FFIEndpoint
    public var paymaster: FFIEndpoint

    public init(rpc: FFIEndpoint,bundler: FFIEndpoint,paymaster: FFIEndpoint) {
        self.rpc = rpc
        self.bundler = bundler
        self.paymaster = paymaster
    }

    @inline(__always)
    func intoFfiRepr() -> __swift_bridge__$FFIEndpoints {
        { let val = self; return __swift_bridge__$FFIEndpoints(rpc: val.rpc.intoFfiRepr(), bundler: val.bundler.intoFfiRepr(), paymaster: val.paymaster.intoFfiRepr()); }()
    }
}
extension __swift_bridge__$FFIEndpoints {
    @inline(__always)
    func intoSwiftRepr() -> FFIEndpoints {
        { let val = self; return FFIEndpoints(rpc: val.rpc.intoSwiftRepr(), bundler: val.bundler.intoSwiftRepr(), paymaster: val.paymaster.intoSwiftRepr()); }()
    }
}
extension __swift_bridge__$Option$FFIEndpoints {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIEndpoints> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }

    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIEndpoints>) -> __swift_bridge__$Option$FFIEndpoints {
        if let v = val {
            return __swift_bridge__$Option$FFIEndpoints(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIEndpoints(is_some: false, val: __swift_bridge__$FFIEndpoints())
        }
    }
}
public struct FFIConfig {
    public var endpoints: FFIEndpoints

    public init(endpoints: FFIEndpoints) {
        self.endpoints = endpoints
    }

    @inline(__always)
    func intoFfiRepr() -> __swift_bridge__$FFIConfig {
        { let val = self; return __swift_bridge__$FFIConfig(endpoints: val.endpoints.intoFfiRepr()); }()
    }
}
extension __swift_bridge__$FFIConfig {
    @inline(__always)
    func intoSwiftRepr() -> FFIConfig {
        { let val = self; return FFIConfig(endpoints: val.endpoints.intoSwiftRepr()); }()
    }
}
extension __swift_bridge__$Option$FFIConfig {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIConfig> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }

    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIConfig>) -> __swift_bridge__$Option$FFIConfig {
        if let v = val {
            return __swift_bridge__$Option$FFIConfig(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIConfig(is_some: false, val: __swift_bridge__$FFIConfig())
        }
    }
}
public struct FFIAccountClientConfig {
    public var owner_address: RustString
    public var chain_id: UInt64
    public var config: FFIConfig
    public var signer_type: RustString
    public var safe: Bool

    public init(owner_address: RustString,chain_id: UInt64,config: FFIConfig,signer_type: RustString,safe: Bool) {
        self.owner_address = owner_address
        self.chain_id = chain_id
        self.config = config
        self.signer_type = signer_type
        self.safe = safe
    }

    @inline(__always)
    func intoFfiRepr() -> __swift_bridge__$FFIAccountClientConfig {
        { let val = self; return __swift_bridge__$FFIAccountClientConfig(owner_address: { let rustString = val.owner_address.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), chain_id: val.chain_id, config: val.config.intoFfiRepr(), signer_type: { let rustString = val.signer_type.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), safe: val.safe); }()
    }
}
extension __swift_bridge__$FFIAccountClientConfig {
    @inline(__always)
    func intoSwiftRepr() -> FFIAccountClientConfig {
        { let val = self; return FFIAccountClientConfig(owner_address: RustString(ptr: val.owner_address), chain_id: val.chain_id, config: val.config.intoSwiftRepr(), signer_type: RustString(ptr: val.signer_type), safe: val.safe); }()
    }
}
extension __swift_bridge__$Option$FFIAccountClientConfig {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIAccountClientConfig> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }

    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIAccountClientConfig>) -> __swift_bridge__$Option$FFIAccountClientConfig {
        if let v = val {
            return __swift_bridge__$Option$FFIAccountClientConfig(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIAccountClientConfig(is_some: false, val: __swift_bridge__$FFIAccountClientConfig())
        }
    }
}
public enum FFIStringResult {
    case Ok(RustString)
    case Err(RustString)
}
extension FFIStringResult {
    func intoFfiRepr() -> __swift_bridge__$FFIStringResult {
        switch self {
            case FFIStringResult.Ok(let _0):
                return __swift_bridge__$FFIStringResult(tag: __swift_bridge__$FFIStringResult$Ok, payload: __swift_bridge__$FFIStringResultFields(Ok: __swift_bridge__$FFIStringResult$FieldOfOk(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
            case FFIStringResult.Err(let _0):
                return __swift_bridge__$FFIStringResult(tag: __swift_bridge__$FFIStringResult$Err, payload: __swift_bridge__$FFIStringResultFields(Err: __swift_bridge__$FFIStringResult$FieldOfErr(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
        }
    }
}
extension __swift_bridge__$FFIStringResult {
    func intoSwiftRepr() -> FFIStringResult {
        switch self.tag {
            case __swift_bridge__$FFIStringResult$Ok:
                return FFIStringResult.Ok(RustString(ptr: self.payload.Ok._0))
            case __swift_bridge__$FFIStringResult$Err:
                return FFIStringResult.Err(RustString(ptr: self.payload.Err._0))
            default:
                fatalError("Unreachable")
        }
    }
}
extension __swift_bridge__$Option$FFIStringResult {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIStringResult> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }
    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIStringResult>) -> __swift_bridge__$Option$FFIStringResult {
        if let v = val {
            return __swift_bridge__$Option$FFIStringResult(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIStringResult(is_some: false, val: __swift_bridge__$FFIStringResult())
        }
    }
}
public enum FFIError {
    case Unknown(RustString)
}
extension FFIError {
    func intoFfiRepr() -> __swift_bridge__$FFIError {
        switch self {
            case FFIError.Unknown(let _0):
                return __swift_bridge__$FFIError(tag: __swift_bridge__$FFIError$Unknown, payload: __swift_bridge__$FFIErrorFields(Unknown: __swift_bridge__$FFIError$FieldOfUnknown(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
        }
    }
}
extension __swift_bridge__$FFIError {
    func intoSwiftRepr() -> FFIError {
        switch self.tag {
            case __swift_bridge__$FFIError$Unknown:
                return FFIError.Unknown(RustString(ptr: self.payload.Unknown._0))
            default:
                fatalError("Unreachable")
        }
    }
}
extension __swift_bridge__$Option$FFIError {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIError> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }
    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIError>) -> __swift_bridge__$Option$FFIError {
        if let v = val {
            return __swift_bridge__$Option$FFIError(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIError(is_some: false, val: __swift_bridge__$FFIError())
        }
    }
}

public class FFIAccountClient: FFIAccountClientRefMut {
    var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$FFIAccountClient$_free(ptr)
        }
    }
}
extension FFIAccountClient {
    public convenience init(_ config: FFIAccountClientConfig) {
        self.init(ptr: __swift_bridge__$FFIAccountClient$new(config.intoFfiRepr()))
    }
}
public class FFIAccountClientRefMut: FFIAccountClientRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class FFIAccountClientRef {
    var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension FFIAccountClientRef {
    public func chain_id() -> UInt64 {
        __swift_bridge__$FFIAccountClient$chain_id(ptr)
    }

    public func get_address() async throws -> RustString {
        func onComplete(cbWrapperPtr: UnsafeMutableRawPointer?, rustFnRetVal: __swift_bridge__$ResultStringAndFFIError) {
            let wrapper = Unmanaged<CbWrapper$FFIAccountClient$get_address>.fromOpaque(cbWrapperPtr!).takeRetainedValue()
            switch rustFnRetVal.tag { case __swift_bridge__$ResultStringAndFFIError$ResultOk: wrapper.cb(.success(RustString(ptr: rustFnRetVal.payload.ok))) case __swift_bridge__$ResultStringAndFFIError$ResultErr: wrapper.cb(.failure(rustFnRetVal.payload.err.intoSwiftRepr())) default: fatalError() }
        }

        return try await withCheckedThrowingContinuation({ (continuation: CheckedContinuation<RustString, Error>) in
            let callback = { rustFnRetVal in
                continuation.resume(with: rustFnRetVal)
            }

            let wrapper = CbWrapper$FFIAccountClient$get_address(cb: callback)
            let wrapperPtr = Unmanaged.passRetained(wrapper).toOpaque()

            __swift_bridge__$FFIAccountClient$get_address(wrapperPtr, onComplete, ptr)
        })
    }
    class CbWrapper$FFIAccountClient$get_address {
        var cb: (Result<RustString, Error>) -> ()
    
        public init(cb: @escaping (Result<RustString, Error>) -> ()) {
            self.cb = cb
        }
    }

    public func send_transactions(_ _transactions: RustVec<FFITransaction>) async throws -> RustString {
        func onComplete(cbWrapperPtr: UnsafeMutableRawPointer?, rustFnRetVal: __swift_bridge__$ResultStringAndFFIError) {
            let wrapper = Unmanaged<CbWrapper$FFIAccountClient$send_transactions>.fromOpaque(cbWrapperPtr!).takeRetainedValue()
            switch rustFnRetVal.tag { case __swift_bridge__$ResultStringAndFFIError$ResultOk: wrapper.cb(.success(RustString(ptr: rustFnRetVal.payload.ok))) case __swift_bridge__$ResultStringAndFFIError$ResultErr: wrapper.cb(.failure(rustFnRetVal.payload.err.intoSwiftRepr())) default: fatalError() }
        }

        return try await withCheckedThrowingContinuation({ (continuation: CheckedContinuation<RustString, Error>) in
            let callback = { rustFnRetVal in
                continuation.resume(with: rustFnRetVal)
            }

            let wrapper = CbWrapper$FFIAccountClient$send_transactions(cb: callback)
            let wrapperPtr = Unmanaged.passRetained(wrapper).toOpaque()

            __swift_bridge__$FFIAccountClient$send_transactions(wrapperPtr, onComplete, ptr, { let val = _transactions; val.isOwned = false; return val.ptr }())
        })
    }
    class CbWrapper$FFIAccountClient$send_transactions {
        var cb: (Result<RustString, Error>) -> ()
    
        public init(cb: @escaping (Result<RustString, Error>) -> ()) {
            self.cb = cb
        }
    }

    public func sign_message_with_mnemonic<GenericIntoRustString: IntoRustString>(_ message: GenericIntoRustString, _ mnemonic: GenericIntoRustString) throws -> RustString {
        try { let val = __swift_bridge__$FFIAccountClient$sign_message_with_mnemonic(ptr, { let rustString = message.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), { let rustString = mnemonic.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); switch val.tag { case __swift_bridge__$ResultStringAndFFIError$ResultOk: return RustString(ptr: val.payload.ok) case __swift_bridge__$ResultStringAndFFIError$ResultErr: throw val.payload.err.intoSwiftRepr() default: fatalError() } }()
    }

    public func wait_for_user_operation_receipt<GenericIntoRustString: IntoRustString>(_ user_operation_hash: GenericIntoRustString) async throws -> RustString {
        func onComplete(cbWrapperPtr: UnsafeMutableRawPointer?, rustFnRetVal: __swift_bridge__$ResultStringAndFFIError) {
            let wrapper = Unmanaged<CbWrapper$FFIAccountClient$wait_for_user_operation_receipt>.fromOpaque(cbWrapperPtr!).takeRetainedValue()
            switch rustFnRetVal.tag { case __swift_bridge__$ResultStringAndFFIError$ResultOk: wrapper.cb(.success(RustString(ptr: rustFnRetVal.payload.ok))) case __swift_bridge__$ResultStringAndFFIError$ResultErr: wrapper.cb(.failure(rustFnRetVal.payload.err.intoSwiftRepr())) default: fatalError() }
        }

        return try await withCheckedThrowingContinuation({ (continuation: CheckedContinuation<RustString, Error>) in
            let callback = { rustFnRetVal in
                continuation.resume(with: rustFnRetVal)
            }

            let wrapper = CbWrapper$FFIAccountClient$wait_for_user_operation_receipt(cb: callback)
            let wrapperPtr = Unmanaged.passRetained(wrapper).toOpaque()

            __swift_bridge__$FFIAccountClient$wait_for_user_operation_receipt(wrapperPtr, onComplete, ptr, { let rustString = user_operation_hash.intoRustString(); rustString.isOwned = false; return rustString.ptr }())
        })
    }
    class CbWrapper$FFIAccountClient$wait_for_user_operation_receipt {
        var cb: (Result<RustString, Error>) -> ()
    
        public init(cb: @escaping (Result<RustString, Error>) -> ()) {
            self.cb = cb
        }
    }
}
extension FFIAccountClient: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_FFIAccountClient$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_FFIAccountClient$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: FFIAccountClient) {
        __swift_bridge__$Vec_FFIAccountClient$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_FFIAccountClient$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (FFIAccountClient(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<FFIAccountClientRef> {
        let pointer = __swift_bridge__$Vec_FFIAccountClient$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return FFIAccountClientRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<FFIAccountClientRefMut> {
        let pointer = __swift_bridge__$Vec_FFIAccountClient$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return FFIAccountClientRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<FFIAccountClientRef> {
        UnsafePointer<FFIAccountClientRef>(OpaquePointer(__swift_bridge__$Vec_FFIAccountClient$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_FFIAccountClient$len(vecPtr)
    }
}


public class FFI7702AccountClient: FFI7702AccountClientRefMut {
    var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$FFI7702AccountClient$_free(ptr)
        }
    }
}
extension FFI7702AccountClient {
    public convenience init(_ config: FFIAccountClientConfig) {
        self.init(ptr: __swift_bridge__$FFI7702AccountClient$new(config.intoFfiRepr()))
    }
}
public class FFI7702AccountClientRefMut: FFI7702AccountClientRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class FFI7702AccountClientRef {
    var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension FFI7702AccountClientRef {
    public func send_batch_transaction<GenericIntoRustString: IntoRustString>(_ batch: GenericIntoRustString) async throws -> RustString {
        func onComplete(cbWrapperPtr: UnsafeMutableRawPointer?, rustFnRetVal: __swift_bridge__$ResultStringAndFFIError) {
            let wrapper = Unmanaged<CbWrapper$FFI7702AccountClient$send_batch_transaction>.fromOpaque(cbWrapperPtr!).takeRetainedValue()
            switch rustFnRetVal.tag { case __swift_bridge__$ResultStringAndFFIError$ResultOk: wrapper.cb(.success(RustString(ptr: rustFnRetVal.payload.ok))) case __swift_bridge__$ResultStringAndFFIError$ResultErr: wrapper.cb(.failure(rustFnRetVal.payload.err.intoSwiftRepr())) default: fatalError() }
        }

        return try await withCheckedThrowingContinuation({ (continuation: CheckedContinuation<RustString, Error>) in
            let callback = { rustFnRetVal in
                continuation.resume(with: rustFnRetVal)
            }

            let wrapper = CbWrapper$FFI7702AccountClient$send_batch_transaction(cb: callback)
            let wrapperPtr = Unmanaged.passRetained(wrapper).toOpaque()

            __swift_bridge__$FFI7702AccountClient$send_batch_transaction(wrapperPtr, onComplete, ptr, { let rustString = batch.intoRustString(); rustString.isOwned = false; return rustString.ptr }())
        })
    }
    class CbWrapper$FFI7702AccountClient$send_batch_transaction {
        var cb: (Result<RustString, Error>) -> ()
    
        public init(cb: @escaping (Result<RustString, Error>) -> ()) {
            self.cb = cb
        }
    }
}
extension FFI7702AccountClient: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_FFI7702AccountClient$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_FFI7702AccountClient$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: FFI7702AccountClient) {
        __swift_bridge__$Vec_FFI7702AccountClient$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_FFI7702AccountClient$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (FFI7702AccountClient(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<FFI7702AccountClientRef> {
        let pointer = __swift_bridge__$Vec_FFI7702AccountClient$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return FFI7702AccountClientRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<FFI7702AccountClientRefMut> {
        let pointer = __swift_bridge__$Vec_FFI7702AccountClient$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return FFI7702AccountClientRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<FFI7702AccountClientRef> {
        UnsafePointer<FFI7702AccountClientRef>(OpaquePointer(__swift_bridge__$Vec_FFI7702AccountClient$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_FFI7702AccountClient$len(vecPtr)
    }
}


@_cdecl("__swift_bridge__$NativeSignerFFI$_free")
func __swift_bridge__NativeSignerFFI__free (ptr: UnsafeMutableRawPointer) {
    let _ = Unmanaged<NativeSignerFFI>.fromOpaque(ptr).takeRetainedValue()
}


@_cdecl("__swift_bridge__$PrivateKeySignerFFI$_free")
func __swift_bridge__PrivateKeySignerFFI__free (ptr: UnsafeMutableRawPointer) {
    let _ = Unmanaged<PrivateKeySignerFFI>.fromOpaque(ptr).takeRetainedValue()
}



