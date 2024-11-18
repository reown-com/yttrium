
import Foundation

class ChainAbstractionClient {
    struct Errors: LocalizedError {
        let message: String

        var errorDescription: String? {
            return message
        }
    }
    public func status(orchestrationId: String) async throws -> StatusResponse {
        do {
            // Call the Rust function
            let jsonString = try await coreAccountClient.status(orchestrationId.intoRustString()).toString()

            // Parse the JSON string into StatusResponse
            let jsonData = Data(jsonString.utf8)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase // Use if your keys are in snake_case
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
}
