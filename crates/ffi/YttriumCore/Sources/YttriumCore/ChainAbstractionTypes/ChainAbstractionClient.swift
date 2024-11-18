
import Foundation

public struct Transaction: Codable {
    public let from: String
    public let to: String
    public let value: String
    public let gas: String
    public let gasPrice: String
    public let data: String
    public let nonce: String
    public let maxFeePerGas: String
    public let maxPriorityFeePerGas: String
    public let chainId: String
}

class ChainAbstractionClient {
    struct Errors: LocalizedError {
        let message: String

        var errorDescription: String? {
            return message
        }
    }

    private let ffiClient: FFIChainClient

    init(ffiClient: FFIChainClient) {
        self.ffiClient = ffiClient
    }

    public func status(orchestrationId: String) async throws -> StatusResponse {
        do {
            // Call the Rust function
            let jsonString = try await ffiClient.status(orchestrationId.intoRustString()).toString()

            // Parse the JSON string into StatusResponse
            let jsonData = Data(jsonString.utf8)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let statusResponse = try decoder.decode(StatusResponse.self, from: jsonData)
            return statusResponse
        } catch let ffiError as FFIError {
            switch ffiError {
            case .Unknown(let x):
                let errorMessage = x.toString()
                throw Errors(message: errorMessage)
            }
        } catch {
            throw error
        }
    }

    public func route(transaction: Transaction) async throws -> RouteResponse {
        do {
            // Encode the transaction to JSON string
            let jsonEncoder = JSONEncoder()
            jsonEncoder.keyEncodingStrategy = .convertToSnakeCase
            let jsonData = try jsonEncoder.encode(transaction)
            guard let jsonString = String(data: jsonData, encoding: .utf8) else {
                throw Errors(message: "Failed to convert Transaction to JSON string")
            }

            // Call the Rust function
            let responseJsonString = try await ffiClient.route(jsonString.intoRustString()).toString()

            // Decode the response JSON string into RouteResponse
            let responseData = Data(responseJsonString.utf8)
            let jsonDecoder = JSONDecoder()
            jsonDecoder.keyDecodingStrategy = .convertFromSnakeCase
            let routeResponse = try jsonDecoder.decode(RouteResponse.self, from: responseData)
            return routeResponse
        } catch let ffiError as FFIRouteError {
            // Map FFIRouteError to Swift error
            switch ffiError {
            case .Request(let message):
                throw Errors(message: "Request error: \(message)")
            case .RequestFailed(let message):
                throw Errors(message: "Request failed: \(message)")
            }
        } catch {
            throw error
        }
    }

}
