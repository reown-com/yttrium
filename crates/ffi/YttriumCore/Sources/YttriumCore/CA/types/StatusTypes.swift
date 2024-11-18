
import Foundation


public struct StatusResponseSuccessPending: Codable {
    public let createdAt: UInt64
    public let checkIn: UInt64
}

public struct StatusResponseSuccessCompleted: Codable {
    public let createdAt: UInt64
}

public struct StatusResponseSuccessError: Codable {
    public let createdAt: UInt64
    public let errorReason: String
}

public struct StatusResponseError: Codable {
    public let error: String
}

public enum StatusResponseSuccess {
    case pending(StatusResponseSuccessPending)
    case completed(StatusResponseSuccessCompleted)
    case error(StatusResponseSuccessError)
}





