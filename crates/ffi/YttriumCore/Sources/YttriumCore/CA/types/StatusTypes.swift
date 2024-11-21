import Foundation

public struct StatusResponsePending: Codable {
    public let createdAt: UInt64
    public let checkIn: UInt64
}

public struct StatusResponseCompleted: Codable {
    public let createdAt: UInt64
}

public struct StatusResponseError: Codable {
    public let createdAt: UInt64
    public let error: String
}

public enum StatusResponse {
    case pending(StatusResponsePending)
    case completed(StatusResponseCompleted)
    case error(StatusResponseError)
}
