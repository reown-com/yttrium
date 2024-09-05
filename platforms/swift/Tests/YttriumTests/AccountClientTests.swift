import Foundation
import XCTest
@testable import YttriumCore
@testable import Yttrium

final class AccountClientTests: XCTestCase {
    
    static let mnemonic = "test test test test test test test test test test test junk"
    
    /// Using `mnemonic` derived at `m/44'/60'/0'/0/0`
    static let privateKeyHex = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    
    static let entryPoint = "0x0000000071727De22E5E9d8BAf0edAc6f37da032"
    
    /// `Ethereum Sepolia` chain ID
    static let chainId = 11155111
    
    static let ownerAddress = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    
    static let simpleAccountAddress = "0x75BD33d92EEAC5Fe41446fcF5953050d691E7fc9"
    
    func testSendTransaction() async throws {
        let config = Config.local()
        
        let accountClient = AccountClient(
            ownerAddress: Self.ownerAddress,
            entryPoint: Self.entryPoint,
            chainId: Self.chainId,
            config: config,
            signerType: .privateKey
        )
        accountClient.register(privateKey: Self.privateKeyHex)
        
        let transaction = Transaction.mock()
        
        let user_operation_hash = try await accountClient.sendTransaction(transaction)
    }
    
    func testGetAddress() async throws {
        let config = Config.local()
        
        let accountClient = AccountClient(
            ownerAddress: Self.ownerAddress,
            entryPoint: Self.entryPoint,
            chainId: Self.chainId,
            config: config,
            signerType: .privateKey
        )
        accountClient.register(privateKey: Self.privateKeyHex)

        let expectedAddress = Self.simpleAccountAddress
        
        let address = try await accountClient.getAddress()
        
        XCTAssertEqual(address, expectedAddress)
    }
}

extension Transaction {
    
    public static func mock() -> Self {
        Self(
            to: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            value: "0",
            data: "0x68656c6c6f"
        )
    }
}
