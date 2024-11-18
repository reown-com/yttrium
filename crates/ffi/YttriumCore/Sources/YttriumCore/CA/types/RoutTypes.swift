import Foundation




public enum RouteResponseSuccess {
    case available(RouteResponseAvailable)
    case notRequired(RouteResponseNotRequired)
}


public struct RouteResponseAvailable: Codable {
    public let orchestrationId: String
    public let transactions: [Transaction]
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
