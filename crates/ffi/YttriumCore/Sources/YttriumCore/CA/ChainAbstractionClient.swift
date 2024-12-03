import Foundation

public struct Eip1559Estimation {
    public var maxFeePerGas: String
    public var maxPriorityFeePerGas: String
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
            let ffiResponse = try await ffiClient.status(orchestrationId)

            // Handle the response
            switch ffiResponse {
            case .Pending(let jsonString):
                // Decode jsonString into StatusResponseSuccessPending
                let data = Data(jsonString.toString().utf8)
                let decoder = JSONDecoder()
                decoder.keyDecodingStrategy = .convertFromSnakeCase
                let pending = try decoder.decode(StatusResponsePending.self, from: data)
                return .pending(pending)
            case .Completed(let jsonString):
                // Decode jsonString into StatusResponseSuccessCompleted
                let data = Data(jsonString.toString().utf8)
                let decoder = JSONDecoder()
                decoder.keyDecodingStrategy = .convertFromSnakeCase
                let completed = try decoder.decode(StatusResponseCompleted.self, from: data)
                return .completed(completed)
            case .Error(let jsonString):
                // Decode jsonString into StatusResponseSuccessError
                let data = Data(jsonString.toString().utf8)
                let decoder = JSONDecoder()
                decoder.keyDecodingStrategy = .convertFromSnakeCase
                let errorResponse = try decoder.decode(StatusResponseError.self, from: data)
                return .error(errorResponse)
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

    public func estimateFees(chainId: String) async throws -> Eip1559Estimation {
           do {
               // Call the Rust function via ffiClient
               let estimation = try await ffiClient.estimate_fees(chainId)

               // Return the estimation directly
               return Eip1559Estimation(
                maxFeePerGas: estimation.maxFeePerGas.toString(),
                maxPriorityFeePerGas: estimation.maxPriorityFeePerGas.toString()
               )
           } catch let error as FFIError {
               // Handle FFIError
               switch error {
               case .Unknown(let message):
                   throw Errors(message: "Unknown error: \(message)")
               }
           } catch {
               // Handle other errors
               throw error
           }
       }

    public func waitForSuccess(orchestrationId: String, checkIn: UInt64) async throws -> StatusResponseCompleted {
           do {
               // Call the Rust function via ffiClient
               let jsonString = try await ffiClient.wait_for_success(
                orchestrationId,
                checkIn
               )

               // Deserialize the JSON string into `StatusResponseCompleted`
               let data = Data(jsonString.toString().utf8)
               let decoder = JSONDecoder()
               decoder.keyDecodingStrategy = .convertFromSnakeCase
               let completed = try decoder.decode(StatusResponseCompleted.self, from: data)

               return completed
           } catch let error as FFIWaitForSuccessError {
               // Handle FFIWaitForSuccessError
               switch error {
               case .StatusResponseError(let jsonString):
                   // Deserialize the error and throw a custom error
                   let data = Data(jsonString.toString().utf8)
                   let decoder = JSONDecoder()
                   decoder.keyDecodingStrategy = .convertFromSnakeCase
                   let statusError = try decoder.decode(StatusResponseError.self, from: data)
                   throw Errors.init(message: statusError.error)
               case .StatusResponsePending(let jsonString):
                   // Deserialize the pending status and throw or handle accordingly
                   let data = Data(jsonString.toString().utf8)
                   let decoder = JSONDecoder()
                   decoder.keyDecodingStrategy = .convertFromSnakeCase
                   let statusPending = try decoder.decode(StatusResponsePending.self, from: data)
                   throw Errors.init(message: "Status response mending")
               case .RouteError(let routeError):
                   // Handle route errors
                   throw Errors.init(message: routeError.toString())
               case .Unknown(let message):
                   throw Errors.init(message: message.toString())
               }
           } catch {
               // Handle any other errors
               throw error
           }
       }
}

