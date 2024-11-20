import Foundation

public enum RouteResponseSuccess {
    case available(RouteResponseAvailable)
    case notRequired(RouteResponseNotRequired)
}

public struct RouteResponseAvailable: Codable {
    public let orchestrationId: String
    public let transactions: [RoutingTransaction]
    public let metadata: Metadata
}

public struct Metadata: Codable {
    public let fundingFrom: [FundingMetadata]
    public let checkIn: UInt64
}

public struct FundingMetadata: Codable {
    public let chainId: String
    public let tokenContract: String
    public let symbol: String
    public let amount: String
}

public struct RouteResponseNotRequired: Codable {
    public let transactions: Empty
}

public struct Empty: Codable {}

public struct RoutingTransaction: Codable {
    public let from: String
    public let to: String
    public let value: String
    public let gas: String
    public let data: String
    public let nonce: String
    public let chainId: String
}
