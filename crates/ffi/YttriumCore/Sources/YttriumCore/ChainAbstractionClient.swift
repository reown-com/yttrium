
import Foundation

public struct EthTransaction: Codable {
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

public final class ChainAbstractionClient {
    struct Errors: LocalizedError {
        let message: String

        var errorDescription: String? {
            return message
        }
    }

    private let ffiClient: FFIChainClient

    public init(projectId: String) {
        self.ffiClient = FFIChainClient(projectId.intoRustString())
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

    public func route(transaction: EthTransaction) async throws -> RouteResponseSuccess {
        // Encode the transaction to JSON string
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let transactionData = try encoder.encode(transaction)
        guard let transactionJsonString = String(data: transactionData, encoding: .utf8) else {
            throw Errors(message: "Failed to encode transaction to JSON string")
        }

        do {
            // Call the Rust function
            let ffiResponse = try await ffiClient.route(transactionJsonString)

            // Handle the response
            switch ffiResponse {
            case .Success(let success):
                switch success {
                case .Available(let jsonString):
                    // Decode jsonString into RouteResponseAvailable
                    let data = Data(jsonString.toString().utf8)
                    let decoder = JSONDecoder()
                    decoder.keyDecodingStrategy = .convertFromSnakeCase
                    let available = try decoder.decode(RouteResponseAvailable.self, from: data)
                    return .available(available)
                case .NotRequired(let jsonString):
                    // Decode jsonString into RouteResponseNotRequired
                    let data = Data(jsonString.toString().utf8)
                    let decoder = JSONDecoder()
                    decoder.keyDecodingStrategy = .convertFromSnakeCase
                    let notRequired = try decoder.decode(RouteResponseNotRequired.self, from: data)
                    return .notRequired(notRequired)
                }
            case .Error(let errorMessage):
                throw Errors(message: errorMessage.toString())
            }
        } catch let ffiError as FFIRouteError {
            // Handle FFIRouteError
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
