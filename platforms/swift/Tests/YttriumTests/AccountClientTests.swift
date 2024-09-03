import Foundation
import XCTest
@testable import YttriumCore
@testable import Yttrium

final class AccountClientTests: XCTestCase {
    func testGetAddress() async throws {
        let accountAddress = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        let ownerAddress = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        let chainId = 0
        let accountClient = AccountClient(
            ownerAddress: ownerAddress, // TODO
            entryPoint: accountAddress, // TODO
            chainId: chainId,
            onSign: { _ in
                fatalError()
            }
        )
        
        let expectedAddress = "EXPECTED_ADDRESS"
        
        let address = try await accountClient.getAddress()
        
        XCTAssertEqual(address, expectedAddress)
    }
}
