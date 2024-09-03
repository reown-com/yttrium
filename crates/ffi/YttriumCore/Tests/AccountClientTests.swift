@testable import YttriumCore
import XCTest

final class AccountClientTests: XCTestCase {

    func testGetAddress() async throws {
        
        let accountAddress = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".intoRustString()
        
        let chainId = Int64(0)
        
        let accountClient = FFIAccountClient(
            .init(
                account_address: accountAddress,
                chain_id: chainId,
                config: .init(
                    endpoints: .init(
                        rpc: .init(
                            api_key: "".intoRustString(),
                            base_url: "https://eth.merkle.io".intoRustString()
                        ),
                        bundler: .init(
                            api_key: "".intoRustString(),
                            base_url: "https://localhost:4337".intoRustString()
                        )
                    )
                )
            )
        )
        
        let expectedAddress = "EXPECTED_ADDRESS"
        
        let address = try await accountClient.get_address().toString()
        
        XCTAssertEqual(address, expectedAddress)
    }
}
