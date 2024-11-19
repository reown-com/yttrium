
import Foundation


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

    public func status(orchestrationId: String) async throws -> StatusResponseSuccess {
        do {
            // Call the Rust function
            let ffiResponse = try await ffiClient.status(orchestrationId)

            // Handle the response
            switch ffiResponse {
            case .Success(let success):
                switch success {
                case .Pending(let jsonString):
                    // Decode jsonString into StatusResponseSuccessPending
                    let data = Data(jsonString.toString().utf8)
                    let decoder = JSONDecoder()
                    decoder.keyDecodingStrategy = .convertFromSnakeCase
                    let pending = try decoder.decode(StatusResponseSuccessPending.self, from: data)
                    return .pending(pending)
                case .Completed(let jsonString):
                    // Decode jsonString into StatusResponseSuccessCompleted
                    let data = Data(jsonString.toString().utf8)
                    let decoder = JSONDecoder()
                    decoder.keyDecodingStrategy = .convertFromSnakeCase
                    let completed = try decoder.decode(StatusResponseSuccessCompleted.self, from: data)
                    return .completed(completed)
                case .Error(let jsonString):
                    // Decode jsonString into StatusResponseSuccessError
                    let data = Data(jsonString.toString().utf8)
                    let decoder = JSONDecoder()
                    decoder.keyDecodingStrategy = .convertFromSnakeCase
                    let errorResponse = try decoder.decode(StatusResponseSuccessError.self, from: data)
                    return .error(errorResponse)
                }
            case .Error(let jsonString):
                // Decode jsonString into StatusResponseError
                let data = Data(jsonString.toString().utf8)
                let decoder = JSONDecoder()
                decoder.keyDecodingStrategy = .convertFromSnakeCase
                let errorResponse = try decoder.decode(StatusResponseError.self, from: data)
                throw Errors(message: errorResponse.error)
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

    public func route(transaction: EthTransaction) async throws -> RouteResponseSuccess {
        do {
            // Call the Rust function
            let ffiResponse = try await ffiClient.route(transaction.ffi())

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
                throw Errors(message: "Request error: \(message.toString())")
            case .RequestFailed(let message):
                throw Errors(message: "Request failed: \(message.toString())")
            }
        } catch {
            throw error
        }
    }

}
