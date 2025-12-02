## Spender Address Metadata (November 13, 2025)

- **Context:** ERC-7730 descriptors do not currently expose spender addresses (e.g., Permit2 `message.spender`) via metadata, so wallets show raw hex in interpolated intents.
- **Decision:** Until the spec grows first-class support, we’ll maintain a separate `clear_signing/assets/address_book.json` registry that seeds the clearsigning address map (starting with Uniswap’s Universal Router on Optimism). Descriptors remain untouched; wallets still fall back to raw hex when an address isn’t in the registry.
- **Implementation:** The resolver builds a per-call address book by combining descriptor metadata (token/info/owner), descriptor-provided `metadata.addressBook` entries, and registry entries keyed by CAIP-10 chain IDs. Both the call and typed-data pipelines receive this merged view so the engine doesn’t need to guess labels or keep its own global cache.
- **Impact:** Keeps the authoritative mapping in our repo without overloading descriptor metadata and lets any clearsigning client pick up new labels by updating the shared JSON.
- **Open question:** whether ERC-7730 should formalize an address-book structure or reference an external registry. Revisit with Ledger/Uniswap once we have more spenders captured.
