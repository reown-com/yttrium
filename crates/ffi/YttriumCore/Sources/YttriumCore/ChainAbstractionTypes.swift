import Foundation

public struct StatusResponseSuccessPending: Codable {
    public let status: String // Should be "Pending"
    public let createdAt: UInt64
    public let checkIn: UInt64
}

public struct StatusResponseSuccessCompleted: Codable {
    public let status: String // Should be "Completed"
    public let createdAt: UInt64
}

public struct StatusResponseSuccessError: Codable {
    public let status: String // Should be "Error"
    public let createdAt: UInt64
    public let errorReason: String
}

public struct StatusResponseError: Codable {
    public let error: String
}

public enum StatusResponseSuccess: Codable {
    case pending(StatusResponseSuccessPending)
    case completed(StatusResponseSuccessCompleted)
    case error(StatusResponseSuccessError)

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if let pending = try? container.decode(StatusResponseSuccessPending.self), pending.status == "Pending" {
            self = .pending(pending)
        } else if let completed = try? container.decode(StatusResponseSuccessCompleted.self), completed.status == "Completed" {
            self = .completed(completed)
        } else if let error = try? container.decode(StatusResponseSuccessError.self), error.status == "Error" {
            self = .error(error)
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unable to decode StatusResponseSuccess")
        }
    }

    public func encode(to encoder: Encoder) throws {
        switch self {
        case .pending(let pending):
            try pending.encode(to: encoder)
        case .completed(let completed):
            try completed.encode(to: encoder)
        case .error(let error):
            try error.encode(to: encoder)
        }
    }
}

public enum StatusResponse: Codable {
    case success(StatusResponseSuccess)
    case error(StatusResponseError)

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if let error = try? container.decode(StatusResponseError.self) {
            self = .error(error)
        } else if let success = try? container.decode(StatusResponseSuccess.self) {
            self = .success(success)
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unable to decode StatusResponse")
        }
    }

    public func encode(to encoder: Encoder) throws {
        switch self {
        case .success(let success):
            try success.encode(to: encoder)
        case .error(let error):
            try error.encode(to: encoder)
        }
    }
}







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
