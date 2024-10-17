import Foundation

public struct Endpoint {
    
    public let baseURL: String
    
    public let apiKey: String?
    
    public init(baseURL: String, apiKey: String? = nil) {
        self.baseURL = baseURL
        self.apiKey = apiKey
    }
}

public struct Endpoints {
    
    public let rpc: Endpoint
    
    public let bundler: Endpoint
    
    public let paymaster: Endpoint
    
    public init(rpc: Endpoint, bundler: Endpoint, paymaster: Endpoint) {
        self.rpc = rpc
        self.bundler = bundler
        self.paymaster = paymaster
    }
}

public struct Config {
    
    public let endpoints: Endpoints
    
    public init(endpoints: Endpoints) {
        self.endpoints = endpoints
    }
}

extension Endpoint {
    
    public static func localRPC() -> Self {
        Self(baseURL: "http://localhost:8545")
    }
    
    public static func localBundler() -> Self {
        Self(baseURL: "http://localhost:4337")
    }
    
    public static func localPaymaster() -> Self {
        Self(baseURL: "http://localhost:3000")
    }
    
    public var ffi: FFIEndpoint {
        FFIEndpoint(
            api_key: (apiKey ?? "").intoRustString(),
            base_url: baseURL.intoRustString()
        )
    }
}

extension Endpoints {
    
    public static func local() -> Self {
        Endpoints(
            rpc: .localRPC(),
            bundler: .localBundler(),
            paymaster: .localPaymaster()
        )
    }
    
    
    public var ffi: FFIEndpoints {
        FFIEndpoints(
            rpc: rpc.ffi,
            bundler: bundler.ffi,
            paymaster: paymaster.ffi
        )
    }
}


extension Config {
    
    public static func local() -> Self {
        Config(endpoints: .local())
    }
    
    public var ffi: FFIConfig {
        FFIConfig(endpoints: endpoints.ffi)
    }
}
