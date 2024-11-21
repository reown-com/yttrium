import RustXcframework
@_cdecl("__swift_bridge__$NativeSignerFFI$new")
public func __swift_bridge__NativeSignerFFI_new (_ signer_id: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(NativeSignerFFI(signer_id: RustString(ptr: signer_id))).toOpaque()
}

@_cdecl("__swift_bridge__$NativeSignerFFI$sign")
public func __swift_bridge__NativeSignerFFI_sign (_ this: UnsafeMutableRawPointer, _ message: UnsafeMutableRawPointer) -> __swift_bridge__$FFIStringResult {
    Unmanaged<NativeSignerFFI>.fromOpaque(this).takeUnretainedValue().sign(message: RustString(ptr: message)).intoFfiRepr()
}

@_cdecl("__swift_bridge__$PrivateKeySignerFFI$new")
public func __swift_bridge__PrivateKeySignerFFI_new (_ signer_id: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(PrivateKeySignerFFI(signer_id: RustString(ptr: signer_id))).toOpaque()
}

@_cdecl("__swift_bridge__$PrivateKeySignerFFI$private_key")
public func __swift_bridge__PrivateKeySignerFFI_private_key (_ this: UnsafeMutableRawPointer) -> __swift_bridge__$FFIStringResult {
    Unmanaged<PrivateKeySignerFFI>.fromOpaque(this).takeUnretainedValue().private_key().intoFfiRepr()
}

public struct FFIEip1559Estimation {
    public var maxFeePerGas: RustString
    public var maxPriorityFeePerGas: RustString

    public init(maxFeePerGas: RustString,maxPriorityFeePerGas: RustString) {
        self.maxFeePerGas = maxFeePerGas
        self.maxPriorityFeePerGas = maxPriorityFeePerGas
    }

    @inline(__always)
    func intoFfiRepr() -> __swift_bridge__$FFIEip1559Estimation {
        { let val = self; return __swift_bridge__$FFIEip1559Estimation(maxFeePerGas: { let rustString = val.maxFeePerGas.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), maxPriorityFeePerGas: { let rustString = val.maxPriorityFeePerGas.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); }()
    }
}
extension __swift_bridge__$FFIEip1559Estimation {
    @inline(__always)
    func intoSwiftRepr() -> FFIEip1559Estimation {
        { let val = self; return FFIEip1559Estimation(maxFeePerGas: RustString(ptr: val.maxFeePerGas), maxPriorityFeePerGas: RustString(ptr: val.maxPriorityFeePerGas)); }()
    }
}
extension __swift_bridge__$Option$FFIEip1559Estimation {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIEip1559Estimation> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }

    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIEip1559Estimation>) -> __swift_bridge__$Option$FFIEip1559Estimation {
        if let v = val {
            return __swift_bridge__$Option$FFIEip1559Estimation(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIEip1559Estimation(is_some: false, val: __swift_bridge__$FFIEip1559Estimation())
        }
    }
}
public enum FFIStatusResponse {
    case Pending(RustString)
    case Completed(RustString)
    case Error(RustString)
}
extension FFIStatusResponse {
    func intoFfiRepr() -> __swift_bridge__$FFIStatusResponse {
        switch self {
            case FFIStatusResponse.Pending(let _0):
                return __swift_bridge__$FFIStatusResponse(tag: __swift_bridge__$FFIStatusResponse$Pending, payload: __swift_bridge__$FFIStatusResponseFields(Pending: __swift_bridge__$FFIStatusResponse$FieldOfPending(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
            case FFIStatusResponse.Completed(let _0):
                return __swift_bridge__$FFIStatusResponse(tag: __swift_bridge__$FFIStatusResponse$Completed, payload: __swift_bridge__$FFIStatusResponseFields(Completed: __swift_bridge__$FFIStatusResponse$FieldOfCompleted(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
            case FFIStatusResponse.Error(let _0):
                return __swift_bridge__$FFIStatusResponse(tag: __swift_bridge__$FFIStatusResponse$Error, payload: __swift_bridge__$FFIStatusResponseFields(Error: __swift_bridge__$FFIStatusResponse$FieldOfError(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
        }
    }
}
extension __swift_bridge__$FFIStatusResponse {
    func intoSwiftRepr() -> FFIStatusResponse {
        switch self.tag {
            case __swift_bridge__$FFIStatusResponse$Pending:
                return FFIStatusResponse.Pending(RustString(ptr: self.payload.Pending._0))
            case __swift_bridge__$FFIStatusResponse$Completed:
                return FFIStatusResponse.Completed(RustString(ptr: self.payload.Completed._0))
            case __swift_bridge__$FFIStatusResponse$Error:
                return FFIStatusResponse.Error(RustString(ptr: self.payload.Error._0))
            default:
                fatalError("Unreachable")
        }
    }
}
extension __swift_bridge__$Option$FFIStatusResponse {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIStatusResponse> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }
    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIStatusResponse>) -> __swift_bridge__$Option$FFIStatusResponse {
        if let v = val {
            return __swift_bridge__$Option$FFIStatusResponse(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIStatusResponse(is_some: false, val: __swift_bridge__$FFIStatusResponse())
        }
    }
}
public enum FFIRouteResponseSuccess {
    case Available(RustString)
    case NotRequired(RustString)
}
extension FFIRouteResponseSuccess {
    func intoFfiRepr() -> __swift_bridge__$FFIRouteResponseSuccess {
        switch self {
            case FFIRouteResponseSuccess.Available(let _0):
                return __swift_bridge__$FFIRouteResponseSuccess(tag: __swift_bridge__$FFIRouteResponseSuccess$Available, payload: __swift_bridge__$FFIRouteResponseSuccessFields(Available: __swift_bridge__$FFIRouteResponseSuccess$FieldOfAvailable(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
            case FFIRouteResponseSuccess.NotRequired(let _0):
                return __swift_bridge__$FFIRouteResponseSuccess(tag: __swift_bridge__$FFIRouteResponseSuccess$NotRequired, payload: __swift_bridge__$FFIRouteResponseSuccessFields(NotRequired: __swift_bridge__$FFIRouteResponseSuccess$FieldOfNotRequired(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
        }
    }
}
extension __swift_bridge__$FFIRouteResponseSuccess {
    func intoSwiftRepr() -> FFIRouteResponseSuccess {
        switch self.tag {
            case __swift_bridge__$FFIRouteResponseSuccess$Available:
                return FFIRouteResponseSuccess.Available(RustString(ptr: self.payload.Available._0))
            case __swift_bridge__$FFIRouteResponseSuccess$NotRequired:
                return FFIRouteResponseSuccess.NotRequired(RustString(ptr: self.payload.NotRequired._0))
            default:
                fatalError("Unreachable")
        }
    }
}
extension __swift_bridge__$Option$FFIRouteResponseSuccess {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIRouteResponseSuccess> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }
    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIRouteResponseSuccess>) -> __swift_bridge__$Option$FFIRouteResponseSuccess {
        if let v = val {
            return __swift_bridge__$Option$FFIRouteResponseSuccess(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIRouteResponseSuccess(is_some: false, val: __swift_bridge__$FFIRouteResponseSuccess())
        }
    }
}
public enum FFIRouteResponse {
    case Success(FFIRouteResponseSuccess)
    case Error(RustString)
}
extension FFIRouteResponse {
    func intoFfiRepr() -> __swift_bridge__$FFIRouteResponse {
        switch self {
            case FFIRouteResponse.Success(let _0):
                return __swift_bridge__$FFIRouteResponse(tag: __swift_bridge__$FFIRouteResponse$Success, payload: __swift_bridge__$FFIRouteResponseFields(Success: __swift_bridge__$FFIRouteResponse$FieldOfSuccess(_0: _0.intoFfiRepr())))
            case FFIRouteResponse.Error(let _0):
                return __swift_bridge__$FFIRouteResponse(tag: __swift_bridge__$FFIRouteResponse$Error, payload: __swift_bridge__$FFIRouteResponseFields(Error: __swift_bridge__$FFIRouteResponse$FieldOfError(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
        }
    }
}
extension __swift_bridge__$FFIRouteResponse {
    func intoSwiftRepr() -> FFIRouteResponse {
        switch self.tag {
            case __swift_bridge__$FFIRouteResponse$Success:
                return FFIRouteResponse.Success(self.payload.Success._0.intoSwiftRepr())
            case __swift_bridge__$FFIRouteResponse$Error:
                return FFIRouteResponse.Error(RustString(ptr: self.payload.Error._0))
            default:
                fatalError("Unreachable")
        }
    }
}
extension __swift_bridge__$Option$FFIRouteResponse {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIRouteResponse> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }
    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIRouteResponse>) -> __swift_bridge__$Option$FFIRouteResponse {
        if let v = val {
            return __swift_bridge__$Option$FFIRouteResponse(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIRouteResponse(is_some: false, val: __swift_bridge__$FFIRouteResponse())
        }
    }
}
public enum FFIRouteError {
    case Request(RustString)
    case RequestFailed(RustString)
}
extension FFIRouteError {
    func intoFfiRepr() -> __swift_bridge__$FFIRouteError {
        switch self {
            case FFIRouteError.Request(let _0):
                return __swift_bridge__$FFIRouteError(tag: __swift_bridge__$FFIRouteError$Request, payload: __swift_bridge__$FFIRouteErrorFields(Request: __swift_bridge__$FFIRouteError$FieldOfRequest(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
            case FFIRouteError.RequestFailed(let _0):
                return __swift_bridge__$FFIRouteError(tag: __swift_bridge__$FFIRouteError$RequestFailed, payload: __swift_bridge__$FFIRouteErrorFields(RequestFailed: __swift_bridge__$FFIRouteError$FieldOfRequestFailed(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
        }
    }
}
extension __swift_bridge__$FFIRouteError {
    func intoSwiftRepr() -> FFIRouteError {
        switch self.tag {
            case __swift_bridge__$FFIRouteError$Request:
                return FFIRouteError.Request(RustString(ptr: self.payload.Request._0))
            case __swift_bridge__$FFIRouteError$RequestFailed:
                return FFIRouteError.RequestFailed(RustString(ptr: self.payload.RequestFailed._0))
            default:
                fatalError("Unreachable")
        }
    }
}
extension __swift_bridge__$Option$FFIRouteError {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIRouteError> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }
    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIRouteError>) -> __swift_bridge__$Option$FFIRouteError {
        if let v = val {
            return __swift_bridge__$Option$FFIRouteError(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIRouteError(is_some: false, val: __swift_bridge__$FFIRouteError())
        }
    }
}
public struct FFIEthTransaction {
    public var from: RustString
    public var to: RustString
    public var value: RustString
    public var gas: RustString
    public var gas_price: RustString
    public var data: RustString
    public var nonce: RustString
    public var max_fee_per_gas: RustString
    public var max_priority_fee_per_gas: RustString
    public var chain_id: RustString

    public init(from: RustString,to: RustString,value: RustString,gas: RustString,gas_price: RustString,data: RustString,nonce: RustString,max_fee_per_gas: RustString,max_priority_fee_per_gas: RustString,chain_id: RustString) {
        self.from = from
        self.to = to
        self.value = value
        self.gas = gas
        self.gas_price = gas_price
        self.data = data
        self.nonce = nonce
        self.max_fee_per_gas = max_fee_per_gas
        self.max_priority_fee_per_gas = max_priority_fee_per_gas
        self.chain_id = chain_id
    }

    @inline(__always)
    func intoFfiRepr() -> __swift_bridge__$FFIEthTransaction {
        { let val = self; return __swift_bridge__$FFIEthTransaction(from: { let rustString = val.from.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), to: { let rustString = val.to.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), value: { let rustString = val.value.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), gas: { let rustString = val.gas.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), gas_price: { let rustString = val.gas_price.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), data: { let rustString = val.data.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), nonce: { let rustString = val.nonce.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), max_fee_per_gas: { let rustString = val.max_fee_per_gas.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), max_priority_fee_per_gas: { let rustString = val.max_priority_fee_per_gas.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), chain_id: { let rustString = val.chain_id.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); }()
    }
}
extension __swift_bridge__$FFIEthTransaction {
    @inline(__always)
    func intoSwiftRepr() -> FFIEthTransaction {
        { let val = self; return FFIEthTransaction(from: RustString(ptr: val.from), to: RustString(ptr: val.to), value: RustString(ptr: val.value), gas: RustString(ptr: val.gas), gas_price: RustString(ptr: val.gas_price), data: RustString(ptr: val.data), nonce: RustString(ptr: val.nonce), max_fee_per_gas: RustString(ptr: val.max_fee_per_gas), max_priority_fee_per_gas: RustString(ptr: val.max_priority_fee_per_gas), chain_id: RustString(ptr: val.chain_id)); }()
    }
}
extension __swift_bridge__$Option$FFIEthTransaction {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIEthTransaction> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }

    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIEthTransaction>) -> __swift_bridge__$Option$FFIEthTransaction {
        if let v = val {
            return __swift_bridge__$Option$FFIEthTransaction(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIEthTransaction(is_some: false, val: __swift_bridge__$FFIEthTransaction())
        }
    }
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
public struct FFIPreparedSign {
    public var signature: RustString
    public var hash: RustString
    public var sign_step_3_params: RustString

    public init(signature: RustString,hash: RustString,sign_step_3_params: RustString) {
        self.signature = signature
        self.hash = hash
        self.sign_step_3_params = sign_step_3_params
    }

    @inline(__always)
    func intoFfiRepr() -> __swift_bridge__$FFIPreparedSign {
        { let val = self; return __swift_bridge__$FFIPreparedSign(signature: { let rustString = val.signature.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), hash: { let rustString = val.hash.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), sign_step_3_params: { let rustString = val.sign_step_3_params.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); }()
    }
}
extension __swift_bridge__$FFIPreparedSign {
    @inline(__always)
    func intoSwiftRepr() -> FFIPreparedSign {
        { let val = self; return FFIPreparedSign(signature: RustString(ptr: val.signature), hash: RustString(ptr: val.hash), sign_step_3_params: RustString(ptr: val.sign_step_3_params)); }()
    }
}
extension __swift_bridge__$Option$FFIPreparedSign {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIPreparedSign> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }

    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIPreparedSign>) -> __swift_bridge__$Option$FFIPreparedSign {
        if let v = val {
            return __swift_bridge__$Option$FFIPreparedSign(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIPreparedSign(is_some: false, val: __swift_bridge__$FFIPreparedSign())
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
public struct FFIPreparedSignature {
    public var hash: RustString

    public init(hash: RustString) {
        self.hash = hash
    }

    @inline(__always)
    func intoFfiRepr() -> __swift_bridge__$FFIPreparedSignature {
        { let val = self; return __swift_bridge__$FFIPreparedSignature(hash: { let rustString = val.hash.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); }()
    }
}
extension __swift_bridge__$FFIPreparedSignature {
    @inline(__always)
    func intoSwiftRepr() -> FFIPreparedSignature {
        { let val = self; return FFIPreparedSignature(hash: RustString(ptr: val.hash)); }()
    }
}
extension __swift_bridge__$Option$FFIPreparedSignature {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIPreparedSignature> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }

    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIPreparedSignature>) -> __swift_bridge__$Option$FFIPreparedSignature {
        if let v = val {
            return __swift_bridge__$Option$FFIPreparedSignature(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIPreparedSignature(is_some: false, val: __swift_bridge__$FFIPreparedSignature())
        }
    }
}
public struct FFIPreparedSendTransaction {
    public var hash: RustString
    public var do_send_transaction_params: RustString

    public init(hash: RustString,do_send_transaction_params: RustString) {
        self.hash = hash
        self.do_send_transaction_params = do_send_transaction_params
    }

    @inline(__always)
    func intoFfiRepr() -> __swift_bridge__$FFIPreparedSendTransaction {
        { let val = self; return __swift_bridge__$FFIPreparedSendTransaction(hash: { let rustString = val.hash.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), do_send_transaction_params: { let rustString = val.do_send_transaction_params.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); }()
    }
}
extension __swift_bridge__$FFIPreparedSendTransaction {
    @inline(__always)
    func intoSwiftRepr() -> FFIPreparedSendTransaction {
        { let val = self; return FFIPreparedSendTransaction(hash: RustString(ptr: val.hash), do_send_transaction_params: RustString(ptr: val.do_send_transaction_params)); }()
    }
}
extension __swift_bridge__$Option$FFIPreparedSendTransaction {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIPreparedSendTransaction> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }

    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIPreparedSendTransaction>) -> __swift_bridge__$Option$FFIPreparedSendTransaction {
        if let v = val {
            return __swift_bridge__$Option$FFIPreparedSendTransaction(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIPreparedSendTransaction(is_some: false, val: __swift_bridge__$FFIPreparedSendTransaction())
        }
    }
}
public struct FFIOwnerSignature {
    public var owner: RustString
    public var signature: RustString

    public init(owner: RustString,signature: RustString) {
        self.owner = owner
        self.signature = signature
    }

    @inline(__always)
    func intoFfiRepr() -> __swift_bridge__$FFIOwnerSignature {
        { let val = self; return __swift_bridge__$FFIOwnerSignature(owner: { let rustString = val.owner.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), signature: { let rustString = val.signature.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); }()
    }
}
extension __swift_bridge__$FFIOwnerSignature {
    @inline(__always)
    func intoSwiftRepr() -> FFIOwnerSignature {
        { let val = self; return FFIOwnerSignature(owner: RustString(ptr: val.owner), signature: RustString(ptr: val.signature)); }()
    }
}
extension __swift_bridge__$Option$FFIOwnerSignature {
    @inline(__always)
    func intoSwiftRepr() -> Optional<FFIOwnerSignature> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }

    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<FFIOwnerSignature>) -> __swift_bridge__$Option$FFIOwnerSignature {
        if let v = val {
            return __swift_bridge__$Option$FFIOwnerSignature(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$FFIOwnerSignature(is_some: false, val: __swift_bridge__$FFIOwnerSignature())
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

    public func prepare_sign_message<GenericIntoRustString: IntoRustString>(_ _message_hash: GenericIntoRustString) async throws -> FFIPreparedSignature {
        func onComplete(cbWrapperPtr: UnsafeMutableRawPointer?, rustFnRetVal: __swift_bridge__$ResultFFIPreparedSignatureAndFFIError) {
            let wrapper = Unmanaged<CbWrapper$FFIAccountClient$prepare_sign_message>.fromOpaque(cbWrapperPtr!).takeRetainedValue()
            switch rustFnRetVal.tag { case __swift_bridge__$ResultFFIPreparedSignatureAndFFIError$ResultOk: wrapper.cb(.success(rustFnRetVal.payload.ok.intoSwiftRepr())) case __swift_bridge__$ResultFFIPreparedSignatureAndFFIError$ResultErr: wrapper.cb(.failure(rustFnRetVal.payload.err.intoSwiftRepr())) default: fatalError() }
        }

        return try await withCheckedThrowingContinuation({ (continuation: CheckedContinuation<FFIPreparedSignature, Error>) in
            let callback = { rustFnRetVal in
                continuation.resume(with: rustFnRetVal)
            }

            let wrapper = CbWrapper$FFIAccountClient$prepare_sign_message(cb: callback)
            let wrapperPtr = Unmanaged.passRetained(wrapper).toOpaque()

            __swift_bridge__$FFIAccountClient$prepare_sign_message(wrapperPtr, onComplete, ptr, { let rustString = _message_hash.intoRustString(); rustString.isOwned = false; return rustString.ptr }())
        })
    }
    class CbWrapper$FFIAccountClient$prepare_sign_message {
        var cb: (Result<FFIPreparedSignature, Error>) -> ()
    
        public init(cb: @escaping (Result<FFIPreparedSignature, Error>) -> ()) {
            self.cb = cb
        }
    }

    public func do_sign_message<GenericIntoRustString: IntoRustString>(_ _signatures: RustVec<GenericIntoRustString>) async throws -> FFIPreparedSign {
        func onComplete(cbWrapperPtr: UnsafeMutableRawPointer?, rustFnRetVal: __swift_bridge__$ResultFFIPreparedSignAndFFIError) {
            let wrapper = Unmanaged<CbWrapper$FFIAccountClient$do_sign_message>.fromOpaque(cbWrapperPtr!).takeRetainedValue()
            switch rustFnRetVal.tag { case __swift_bridge__$ResultFFIPreparedSignAndFFIError$ResultOk: wrapper.cb(.success(rustFnRetVal.payload.ok.intoSwiftRepr())) case __swift_bridge__$ResultFFIPreparedSignAndFFIError$ResultErr: wrapper.cb(.failure(rustFnRetVal.payload.err.intoSwiftRepr())) default: fatalError() }
        }

        return try await withCheckedThrowingContinuation({ (continuation: CheckedContinuation<FFIPreparedSign, Error>) in
            let callback = { rustFnRetVal in
                continuation.resume(with: rustFnRetVal)
            }

            let wrapper = CbWrapper$FFIAccountClient$do_sign_message(cb: callback)
            let wrapperPtr = Unmanaged.passRetained(wrapper).toOpaque()

            __swift_bridge__$FFIAccountClient$do_sign_message(wrapperPtr, onComplete, ptr, { let val = _signatures; val.isOwned = false; return val.ptr }())
        })
    }
    class CbWrapper$FFIAccountClient$do_sign_message {
        var cb: (Result<FFIPreparedSign, Error>) -> ()
    
        public init(cb: @escaping (Result<FFIPreparedSign, Error>) -> ()) {
            self.cb = cb
        }
    }

    public func finalize_sign_message<GenericIntoRustString: IntoRustString>(_ signatures: RustVec<GenericIntoRustString>, _ sign_step_3_params: GenericIntoRustString) async throws -> RustString {
        func onComplete(cbWrapperPtr: UnsafeMutableRawPointer?, rustFnRetVal: __swift_bridge__$ResultStringAndFFIError) {
            let wrapper = Unmanaged<CbWrapper$FFIAccountClient$finalize_sign_message>.fromOpaque(cbWrapperPtr!).takeRetainedValue()
            switch rustFnRetVal.tag { case __swift_bridge__$ResultStringAndFFIError$ResultOk: wrapper.cb(.success(RustString(ptr: rustFnRetVal.payload.ok))) case __swift_bridge__$ResultStringAndFFIError$ResultErr: wrapper.cb(.failure(rustFnRetVal.payload.err.intoSwiftRepr())) default: fatalError() }
        }

        return try await withCheckedThrowingContinuation({ (continuation: CheckedContinuation<RustString, Error>) in
            let callback = { rustFnRetVal in
                continuation.resume(with: rustFnRetVal)
            }

            let wrapper = CbWrapper$FFIAccountClient$finalize_sign_message(cb: callback)
            let wrapperPtr = Unmanaged.passRetained(wrapper).toOpaque()

            __swift_bridge__$FFIAccountClient$finalize_sign_message(wrapperPtr, onComplete, ptr, { let val = signatures; val.isOwned = false; return val.ptr }(), { let rustString = sign_step_3_params.intoRustString(); rustString.isOwned = false; return rustString.ptr }())
        })
    }
    class CbWrapper$FFIAccountClient$finalize_sign_message {
        var cb: (Result<RustString, Error>) -> ()
    
        public init(cb: @escaping (Result<RustString, Error>) -> ()) {
            self.cb = cb
        }
    }

    public func send_transactions<GenericIntoRustString: IntoRustString>(_ _transactions: RustVec<GenericIntoRustString>) async throws -> RustString {
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

    public func prepare_send_transactions<GenericIntoRustString: IntoRustString>(_ _transactions: RustVec<GenericIntoRustString>) async throws -> FFIPreparedSendTransaction {
        func onComplete(cbWrapperPtr: UnsafeMutableRawPointer?, rustFnRetVal: __swift_bridge__$ResultFFIPreparedSendTransactionAndFFIError) {
            let wrapper = Unmanaged<CbWrapper$FFIAccountClient$prepare_send_transactions>.fromOpaque(cbWrapperPtr!).takeRetainedValue()
            switch rustFnRetVal.tag { case __swift_bridge__$ResultFFIPreparedSendTransactionAndFFIError$ResultOk: wrapper.cb(.success(rustFnRetVal.payload.ok.intoSwiftRepr())) case __swift_bridge__$ResultFFIPreparedSendTransactionAndFFIError$ResultErr: wrapper.cb(.failure(rustFnRetVal.payload.err.intoSwiftRepr())) default: fatalError() }
        }

        return try await withCheckedThrowingContinuation({ (continuation: CheckedContinuation<FFIPreparedSendTransaction, Error>) in
            let callback = { rustFnRetVal in
                continuation.resume(with: rustFnRetVal)
            }

            let wrapper = CbWrapper$FFIAccountClient$prepare_send_transactions(cb: callback)
            let wrapperPtr = Unmanaged.passRetained(wrapper).toOpaque()

            __swift_bridge__$FFIAccountClient$prepare_send_transactions(wrapperPtr, onComplete, ptr, { let val = _transactions; val.isOwned = false; return val.ptr }())
        })
    }
    class CbWrapper$FFIAccountClient$prepare_send_transactions {
        var cb: (Result<FFIPreparedSendTransaction, Error>) -> ()
    
        public init(cb: @escaping (Result<FFIPreparedSendTransaction, Error>) -> ()) {
            self.cb = cb
        }
    }

    public func do_send_transaction<GenericIntoRustString: IntoRustString>(_ _signatures: RustVec<GenericIntoRustString>, _ _do_send_transaction_params: GenericIntoRustString) async throws -> RustString {
        func onComplete(cbWrapperPtr: UnsafeMutableRawPointer?, rustFnRetVal: __swift_bridge__$ResultStringAndFFIError) {
            let wrapper = Unmanaged<CbWrapper$FFIAccountClient$do_send_transaction>.fromOpaque(cbWrapperPtr!).takeRetainedValue()
            switch rustFnRetVal.tag { case __swift_bridge__$ResultStringAndFFIError$ResultOk: wrapper.cb(.success(RustString(ptr: rustFnRetVal.payload.ok))) case __swift_bridge__$ResultStringAndFFIError$ResultErr: wrapper.cb(.failure(rustFnRetVal.payload.err.intoSwiftRepr())) default: fatalError() }
        }

        return try await withCheckedThrowingContinuation({ (continuation: CheckedContinuation<RustString, Error>) in
            let callback = { rustFnRetVal in
                continuation.resume(with: rustFnRetVal)
            }

            let wrapper = CbWrapper$FFIAccountClient$do_send_transaction(cb: callback)
            let wrapperPtr = Unmanaged.passRetained(wrapper).toOpaque()

            __swift_bridge__$FFIAccountClient$do_send_transaction(wrapperPtr, onComplete, ptr, { let val = _signatures; val.isOwned = false; return val.ptr }(), { let rustString = _do_send_transaction_params.intoRustString(); rustString.isOwned = false; return rustString.ptr }())
        })
    }
    class CbWrapper$FFIAccountClient$do_send_transaction {
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
public func __swift_bridge__NativeSignerFFI__free (ptr: UnsafeMutableRawPointer) {
    let _ = Unmanaged<NativeSignerFFI>.fromOpaque(ptr).takeRetainedValue()
}


@_cdecl("__swift_bridge__$PrivateKeySignerFFI$_free")
public func __swift_bridge__PrivateKeySignerFFI__free (ptr: UnsafeMutableRawPointer) {
    let _ = Unmanaged<PrivateKeySignerFFI>.fromOpaque(ptr).takeRetainedValue()
}

public enum Erc6492Error {
    case InvalidSignature(RustString)
    case InvalidAddress(RustString)
    case InvalidMessageHash(RustString)
    case Verification(RustString)
}
extension Erc6492Error {
    func intoFfiRepr() -> __swift_bridge__$Erc6492Error {
        switch self {
            case Erc6492Error.InvalidSignature(let _0):
                return __swift_bridge__$Erc6492Error(tag: __swift_bridge__$Erc6492Error$InvalidSignature, payload: __swift_bridge__$Erc6492ErrorFields(InvalidSignature: __swift_bridge__$Erc6492Error$FieldOfInvalidSignature(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
            case Erc6492Error.InvalidAddress(let _0):
                return __swift_bridge__$Erc6492Error(tag: __swift_bridge__$Erc6492Error$InvalidAddress, payload: __swift_bridge__$Erc6492ErrorFields(InvalidAddress: __swift_bridge__$Erc6492Error$FieldOfInvalidAddress(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
            case Erc6492Error.InvalidMessageHash(let _0):
                return __swift_bridge__$Erc6492Error(tag: __swift_bridge__$Erc6492Error$InvalidMessageHash, payload: __swift_bridge__$Erc6492ErrorFields(InvalidMessageHash: __swift_bridge__$Erc6492Error$FieldOfInvalidMessageHash(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
            case Erc6492Error.Verification(let _0):
                return __swift_bridge__$Erc6492Error(tag: __swift_bridge__$Erc6492Error$Verification, payload: __swift_bridge__$Erc6492ErrorFields(Verification: __swift_bridge__$Erc6492Error$FieldOfVerification(_0: { let rustString = _0.intoRustString(); rustString.isOwned = false; return rustString.ptr }())))
        }
    }
}
extension __swift_bridge__$Erc6492Error {
    func intoSwiftRepr() -> Erc6492Error {
        switch self.tag {
            case __swift_bridge__$Erc6492Error$InvalidSignature:
                return Erc6492Error.InvalidSignature(RustString(ptr: self.payload.InvalidSignature._0))
            case __swift_bridge__$Erc6492Error$InvalidAddress:
                return Erc6492Error.InvalidAddress(RustString(ptr: self.payload.InvalidAddress._0))
            case __swift_bridge__$Erc6492Error$InvalidMessageHash:
                return Erc6492Error.InvalidMessageHash(RustString(ptr: self.payload.InvalidMessageHash._0))
            case __swift_bridge__$Erc6492Error$Verification:
                return Erc6492Error.Verification(RustString(ptr: self.payload.Verification._0))
            default:
                fatalError("Unreachable")
        }
    }
}
extension __swift_bridge__$Option$Erc6492Error {
    @inline(__always)
    func intoSwiftRepr() -> Optional<Erc6492Error> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }
    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<Erc6492Error>) -> __swift_bridge__$Option$Erc6492Error {
        if let v = val {
            return __swift_bridge__$Option$Erc6492Error(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$Erc6492Error(is_some: false, val: __swift_bridge__$Erc6492Error())
        }
    }
}

public class Erc6492Client: Erc6492ClientRefMut {
    var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$Erc6492Client$_free(ptr)
        }
    }
}
extension Erc6492Client {
    public convenience init<GenericIntoRustString: IntoRustString>(_ rpc_url: GenericIntoRustString) {
        self.init(ptr: __swift_bridge__$Erc6492Client$new({ let rustString = rpc_url.intoRustString(); rustString.isOwned = false; return rustString.ptr }()))
    }
}
public class Erc6492ClientRefMut: Erc6492ClientRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class Erc6492ClientRef {
    var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension Erc6492ClientRef {
    public func verify_signature<GenericIntoRustString: IntoRustString>(_ signature: GenericIntoRustString, _ address: GenericIntoRustString, _ message_hash: GenericIntoRustString) async throws -> Bool {
        func onComplete(cbWrapperPtr: UnsafeMutableRawPointer?, rustFnRetVal: __swift_bridge__$ResultBoolAndErc6492Error) {
            let wrapper = Unmanaged<CbWrapper$Erc6492Client$verify_signature>.fromOpaque(cbWrapperPtr!).takeRetainedValue()
            switch rustFnRetVal.tag { case __swift_bridge__$ResultBoolAndErc6492Error$ResultOk: wrapper.cb(.success(rustFnRetVal.payload.ok)) case __swift_bridge__$ResultBoolAndErc6492Error$ResultErr: wrapper.cb(.failure(rustFnRetVal.payload.err.intoSwiftRepr())) default: fatalError() }
        }

        return try await withCheckedThrowingContinuation({ (continuation: CheckedContinuation<Bool, Error>) in
            let callback = { rustFnRetVal in
                continuation.resume(with: rustFnRetVal)
            }

            let wrapper = CbWrapper$Erc6492Client$verify_signature(cb: callback)
            let wrapperPtr = Unmanaged.passRetained(wrapper).toOpaque()

            __swift_bridge__$Erc6492Client$verify_signature(wrapperPtr, onComplete, ptr, { let rustString = signature.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), { let rustString = address.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), { let rustString = message_hash.intoRustString(); rustString.isOwned = false; return rustString.ptr }())
        })
    }
    class CbWrapper$Erc6492Client$verify_signature {
        var cb: (Result<Bool, Error>) -> ()
    
        public init(cb: @escaping (Result<Bool, Error>) -> ()) {
            self.cb = cb
        }
    }
}
extension Erc6492Client: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_Erc6492Client$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_Erc6492Client$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: Erc6492Client) {
        __swift_bridge__$Vec_Erc6492Client$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_Erc6492Client$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (Erc6492Client(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<Erc6492ClientRef> {
        let pointer = __swift_bridge__$Vec_Erc6492Client$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return Erc6492ClientRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<Erc6492ClientRefMut> {
        let pointer = __swift_bridge__$Vec_Erc6492Client$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return Erc6492ClientRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<Erc6492ClientRef> {
        UnsafePointer<Erc6492ClientRef>(OpaquePointer(__swift_bridge__$Vec_Erc6492Client$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_Erc6492Client$len(vecPtr)
    }
}


public class FFIChainClient: FFIChainClientRefMut {
    var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$FFIChainClient$_free(ptr)
        }
    }
}
extension FFIChainClient {
    public convenience init<GenericIntoRustString: IntoRustString>(_ project_id: GenericIntoRustString) {
        self.init(ptr: __swift_bridge__$FFIChainClient$new({ let rustString = project_id.intoRustString(); rustString.isOwned = false; return rustString.ptr }()))
    }
}
public class FFIChainClientRefMut: FFIChainClientRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class FFIChainClientRef {
    var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension FFIChainClientRef {
    public func route(_ transaction: FFIEthTransaction) async throws -> FFIRouteResponse {
        func onComplete(cbWrapperPtr: UnsafeMutableRawPointer?, rustFnRetVal: __swift_bridge__$ResultFFIRouteResponseAndFFIRouteError) {
            let wrapper = Unmanaged<CbWrapper$FFIChainClient$route>.fromOpaque(cbWrapperPtr!).takeRetainedValue()
            switch rustFnRetVal.tag { case __swift_bridge__$ResultFFIRouteResponseAndFFIRouteError$ResultOk: wrapper.cb(.success(rustFnRetVal.payload.ok.intoSwiftRepr())) case __swift_bridge__$ResultFFIRouteResponseAndFFIRouteError$ResultErr: wrapper.cb(.failure(rustFnRetVal.payload.err.intoSwiftRepr())) default: fatalError() }
        }

        return try await withCheckedThrowingContinuation({ (continuation: CheckedContinuation<FFIRouteResponse, Error>) in
            let callback = { rustFnRetVal in
                continuation.resume(with: rustFnRetVal)
            }

            let wrapper = CbWrapper$FFIChainClient$route(cb: callback)
            let wrapperPtr = Unmanaged.passRetained(wrapper).toOpaque()

            __swift_bridge__$FFIChainClient$route(wrapperPtr, onComplete, ptr, transaction.intoFfiRepr())
        })
    }
    class CbWrapper$FFIChainClient$route {
        var cb: (Result<FFIRouteResponse, Error>) -> ()
    
        public init(cb: @escaping (Result<FFIRouteResponse, Error>) -> ()) {
            self.cb = cb
        }
    }

    public func status<GenericIntoRustString: IntoRustString>(_ orchestration_id: GenericIntoRustString) async throws -> FFIStatusResponse {
        func onComplete(cbWrapperPtr: UnsafeMutableRawPointer?, rustFnRetVal: __swift_bridge__$ResultFFIStatusResponseAndFFIRouteError) {
            let wrapper = Unmanaged<CbWrapper$FFIChainClient$status>.fromOpaque(cbWrapperPtr!).takeRetainedValue()
            switch rustFnRetVal.tag { case __swift_bridge__$ResultFFIStatusResponseAndFFIRouteError$ResultOk: wrapper.cb(.success(rustFnRetVal.payload.ok.intoSwiftRepr())) case __swift_bridge__$ResultFFIStatusResponseAndFFIRouteError$ResultErr: wrapper.cb(.failure(rustFnRetVal.payload.err.intoSwiftRepr())) default: fatalError() }
        }

        return try await withCheckedThrowingContinuation({ (continuation: CheckedContinuation<FFIStatusResponse, Error>) in
            let callback = { rustFnRetVal in
                continuation.resume(with: rustFnRetVal)
            }

            let wrapper = CbWrapper$FFIChainClient$status(cb: callback)
            let wrapperPtr = Unmanaged.passRetained(wrapper).toOpaque()

            __swift_bridge__$FFIChainClient$status(wrapperPtr, onComplete, ptr, { let rustString = orchestration_id.intoRustString(); rustString.isOwned = false; return rustString.ptr }())
        })
    }
    class CbWrapper$FFIChainClient$status {
        var cb: (Result<FFIStatusResponse, Error>) -> ()
    
        public init(cb: @escaping (Result<FFIStatusResponse, Error>) -> ()) {
            self.cb = cb
        }
    }

    public func estimate_fees<GenericIntoRustString: IntoRustString>(_ chain_id: GenericIntoRustString) async throws -> FFIEip1559Estimation {
        func onComplete(cbWrapperPtr: UnsafeMutableRawPointer?, rustFnRetVal: __swift_bridge__$ResultFFIEip1559EstimationAndFFIError) {
            let wrapper = Unmanaged<CbWrapper$FFIChainClient$estimate_fees>.fromOpaque(cbWrapperPtr!).takeRetainedValue()
            switch rustFnRetVal.tag { case __swift_bridge__$ResultFFIEip1559EstimationAndFFIError$ResultOk: wrapper.cb(.success(rustFnRetVal.payload.ok.intoSwiftRepr())) case __swift_bridge__$ResultFFIEip1559EstimationAndFFIError$ResultErr: wrapper.cb(.failure(rustFnRetVal.payload.err.intoSwiftRepr())) default: fatalError() }
        }

        return try await withCheckedThrowingContinuation({ (continuation: CheckedContinuation<FFIEip1559Estimation, Error>) in
            let callback = { rustFnRetVal in
                continuation.resume(with: rustFnRetVal)
            }

            let wrapper = CbWrapper$FFIChainClient$estimate_fees(cb: callback)
            let wrapperPtr = Unmanaged.passRetained(wrapper).toOpaque()

            __swift_bridge__$FFIChainClient$estimate_fees(wrapperPtr, onComplete, ptr, { let rustString = chain_id.intoRustString(); rustString.isOwned = false; return rustString.ptr }())
        })
    }
    class CbWrapper$FFIChainClient$estimate_fees {
        var cb: (Result<FFIEip1559Estimation, Error>) -> ()
    
        public init(cb: @escaping (Result<FFIEip1559Estimation, Error>) -> ()) {
            self.cb = cb
        }
    }
}
extension FFIChainClient: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_FFIChainClient$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_FFIChainClient$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: FFIChainClient) {
        __swift_bridge__$Vec_FFIChainClient$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_FFIChainClient$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (FFIChainClient(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<FFIChainClientRef> {
        let pointer = __swift_bridge__$Vec_FFIChainClient$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return FFIChainClientRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<FFIChainClientRefMut> {
        let pointer = __swift_bridge__$Vec_FFIChainClient$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return FFIChainClientRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<FFIChainClientRef> {
        UnsafePointer<FFIChainClientRef>(OpaquePointer(__swift_bridge__$Vec_FFIChainClient$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_FFIChainClient$len(vecPtr)
    }
}



