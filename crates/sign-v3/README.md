# sign-v3

A WalletConnect Sign Protocol v3 client in Rust. One crate serves every platform:

| Surface | For | Entry point |
|---|---|---|
| Rust API | Rust apps and other crates | `SignClient` |
| uniffi | Swift and Kotlin | `SignClientFfi` |
| wasm-bindgen | Browsers and Node | `WasmSignClient` |

All three share one set of methods, arguments, results, events and errors. Namespaces, proposals, requests, responses and events keep Sign v2's shapes.

**Status: experimental.** It speaks v3 to a v3 Relay only; it doesn't talk to v2 peers.

- [Quick start](#quick-start)
- [Crate layout](#crate-layout)
- [How it works](#how-it-works)
- [Build](#build)
- [Use](#use)
- [Test](#test)
- [Limits and open work](#limits-and-open-work)
- [WASM size](#wasm-size)

## Quick start

```bash
cargo build -p sign-v3                    # native, reqwest without TLS
cargo test -p sign-v3 --lib               # unit tests, offline
SIGN_V3_RELAY_HOSTS=https://<v3 relay> cargo test -p sign-v3 --features rustls   # end to end
```

```rust
use sign_v3::{MemoryStorage, SignClient, SignConfig};

let client = SignClient::init(config, storage, listener).await?; // restores identity and sessions
client.start();                                                  // starts the event poll
let connected = client.connect(params, None).await?;             // dApp: show connected.uri as a QR
```

## Crate layout

The modules depend in one direction: `protocol` ← `transport` ← `client` ← `ffi`. `host` holds the traits the platform implements.

| Path | What it is |
|---|---|
| `src/lib.rs` | Module list and re-exports of the public types |
| **`src/protocol/`** | **Pure, synchronous protocol code. Imports nothing else from the crate.** |
| `protocol/constants.rs` | Time windows, caps and limits |
| `protocol/types.rs` | Wire types: modes and intents, tags and who may send them, signed objects, messages, Relay events, poll config |
| `protocol/ijson.rs` | Strict I-JSON parser (RFC 7493), the crate's only JSON entry point |
| `protocol/encoding.rs` | base64url |
| `protocol/crypto.rs` | Ed25519 with strict verification, the bindingKey, pairing keys, sealing keys, XChaCha20-Poly1305 |
| `protocol/message.rs` | Signing labels, signed objects, building, reading, verifying and opening messages |
| `protocol/intent.rs` | Monitored or sealed: checks the wallet's choice against the dApp's |
| `protocol/uri.rs` | The `wc:` URI: builds and parses it, and tells it apart from a v2 URI |
| `protocol/replay.rs` | The receive-side replay window over `seq` |
| `protocol/mod.rs` | `ProtocolError`, and the test vectors and negative vectors |
| **`src/transport/`** | **HTTP** |
| `transport/mod.rs` | The `HttpClient` trait: a request/response call and a streamed response |
| `transport/native.rs` | reqwest implementation |
| `transport/wasm.rs` | `fetch` implementation, aborting on drop |
| `transport/relay.rs` | Relay calls with retries and backoff, the client JWT, the poll request, and the poll config with local bounds |
| **`src/client/`** | **The client** |
| `client/mod.rs` | `SignClient`: public methods, the send pipeline, event handling, local expiry |
| `client/poller.rs` | The receive path: one streamed poll at a time, liveness, host failover |
| `client/state.rs` | Persisted records and their storage keys |
| `client/types.rs` | Public types: config, namespaces, proposals, sessions, requests, events, errors |
| **`src/host.rs`** | **What the platform implements: `Storage` and `Listener`. `MemoryStorage` is for tests.** |
| **`src/rt.rs`** | **Runtime glue that works on native and WASM: spawn, sleep, clock, boxed futures** |
| **`src/ffi/`** | **Bindings** |
| `ffi/uniffi_api.rs` | `SignClientFfi`, the typed uniffi API |
| `ffi/wasm.rs` | `WasmSignClient`, with JS storage and listener adapters |
| `ffi/json.rs` | The JSON facade, `call(method, paramsJson)`, shared by the web and React Native |
| `src/bin/uniffi_bindgen.rs` | The uniffi bindings generator |
| **`tests/`** | **Integration tests against a running Relay. See [Test](#test).** |

## How it works

### Connecting

1. **The dApp calls `connect`.**
   - It makes a fresh symKey. The bindingKey, an Ed25519 key, is derived from it.
   - It signs a bindingProof and the proposal.
   - It creates a pending connection at the Relay, which returns a `connectionId`.
   - The URI is v2's, with the `connectionId` where the topic was: `wc:<connectionId>@2?relay-protocol=irn&symKey=…&expiryTimestamp=…`.
2. **The wallet calls `pair`.**
   - It parses the URI, derives the same bindingKey, and fetches the proposal.
   - It verifies the bindingProof and the proposal's signature. A request that fails any check is never shown.
3. **The wallet calls `approve` or `reject`.** It signs a binding (or a rejection) naming the `connectionId`.
4. **The dApp verifies the binding,** then emits `session_connect`. A binding that fails is closed with a signed close.

The `connectionId` is the session's topic from then on.

### Messages

- **Format.** A message is `{ plain, payload, sig }`. `plain` holds the tag, `seq`, deadline and sealing flag. The signature covers a label, the bindingPub, `plain` and the payload.
- **Monitored or sealed.** Monitored payloads are plain JSON-RPC that the Relay can read. Sealed payloads are XChaCha20-Poly1305 under per-direction keys derived from both pairing keys. Sealing needs the dApp project's entitlement.
- **`seq`.**
  - Each direction counts up with every message.
  - It's persisted before signing and never reused.
  - Receivers drop repeats through a replay window and accept gaps.
- **Deadlines and `ttl`.** A message carries a deadline. The Relay keeps it for a `ttl`:
  - normally `deadline - now`;
  - for an update or an extension, until the session it changes would end, so a peer that's offline for days still applies it.
- **Retries.** A send is written to an outbox before it's posted, and resent with identical bytes after a crash.
- **Requests are kept on both sides,** so a restart doesn't lose them:
  - the wallet stores each incoming request, so `respond` works after a restart;
  - the dApp stores each outgoing request before it can be delivered, so a late response or expiry still matches it.

### Receiving

- **One long poll per client.**
  - The client sends `POST /v3/events` with its cursor and a counter `n`.
  - The Relay streams NDJSON: a config line, heartbeats, then the events.
  - The client handles the events, persists the cursor, and polls again.
- **Liveness and failover.**
  - Silence past the configured window counts as a dead connection.
  - The client moves to the next host, backing off as the Relay's config says.
  - `foreground()` runs the liveness check at once.
- **Several tabs or processes on one identity.** The highest `n` wins; an older poll ends as `superseded`.
  - **Native:** the superseded client resumes on its next foreground.
  - **Web:** a superseded tab reloads its state and resumes on its next call.
- **Expiry is decided locally,** after a poll that caught up, or on a heartbeat:
  - an unanswered connection ends 30 s after its window;
  - a call ends 30 s after its deadline;
  - a session ends at its expiry.

### Storage

The host gives the client an async key-value store. Every key is prefixed with `storagePrefix`, `wc3:` by default.

| Key | Holds |
|---|---|
| `secret/client` | The client's identity key, for the Relay JWT |
| `conn/{id}`, `secret/conn/{id}` | A session; its pairing key and sealing keys |
| `req/{id}`, `secret/req/{id}` | A dApp's pending connection; its symKey and pairing key |
| `prop/{id}`, `secret/prop/{id}` | A wallet's received proposal; its symKey and pairing key |
| `out/{conn}/{seq}` | A signed message not yet acknowledged by the Relay |
| `inreq/{conn}/{rpcId}` | A received request not yet answered |
| `outreq/{conn}/{rpcId}` | A sent request awaiting its response |
| `poll/cursor`, `poll/n` | The poll cursor and counter |

Keys under `secret/` hold private keys and must go to the platform's secure store: Keychain, Keystore, or encrypted IndexedDB. Writes must be durable once `set` returns; no transactions are needed.

### The Relay API

| Call | Request |
|---|---|
| Create a connection | `POST /v3/connections` returns `{ connectionId, expiresAt }` |
| Fetch it | `GET /v3/connections/{id}` |
| Answer it | `PUT /v3/connections/{id}/acceptance` or `/rejection` |
| Send | `POST /v3/connections/{id}/messages` with `{ message, ttl }` |
| Close | `DELETE /v3/connections/{id}` with a `WC-Signature` header |
| Poll | `POST /v3/events` |

Every call carries `Authorization: Bearer <JWT>`, signed with the client's Ed25519 identity key as a `did:key`, and `?projectId=…`.

## Build

| Feature | Gives |
|---|---|
| `native` (default) | reqwest and tokio |
| `rustls`, `native-tls` | `native` with TLS. Use one for an `https://` Relay. |
| `uniffi` | The uniffi exports |
| `bindgen` | `uniffi` plus the bindings generator binary |
| `wasm` | wasm-bindgen exports. Build with `--no-default-features`. |

```bash
cargo build -p sign-v3 --features rustls                       # native
cargo build -p sign-v3 --features uniffi --lib                 # uniffi library
cargo build -p sign-v3 --release --target wasm32-unknown-unknown \
  --no-default-features --features wasm                        # browser
```

**Swift or Kotlin bindings:**

```bash
cargo build -p sign-v3 --features uniffi --lib
cargo run -p sign-v3 --features bindgen --bin sign-v3-uniffi-bindgen -- \
  generate --library target/debug/libsign_v3.dylib --language swift --out-dir out/swift   # or --language kotlin
```

**JS package for the web:**

```bash
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/sign_v3.wasm
wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
  pkg/sign_v3_bg.wasm -o pkg/sign_v3_bg.wasm
```

The `wasm-bindgen` CLI must match the crate's wasm-bindgen version, 0.2.114 at the time of writing. For the smaller nightly build, see [WASM size](#wasm-size).

## Use

### Setup and lifecycle

The host provides three things:
- **Config.** Relay hosts, primary first; the `projectId`; the app's metadata.
- **Storage.** Async `get`, `set`, `remove` and `keys(prefix)`. See [Storage](#storage).
- **Listener.** Receives every event.

| Call | When |
|---|---|
| `init(config, storage, listener)` | Once. Loads or creates the client identity and restores sessions. |
| `start()` | After the listener is in place. Starts the poll that delivers events. |
| `foreground()` | On app foreground, network online and `pageshow`. Checks liveness at once. |
| `stop()` | Optional, on app background. Call `start()` again on return. |

Config, as JSON for the web and React Native. On native, the same fields make up the `SignConfig` record:

```json
{
  "hosts": ["https://<v3 relay>"],
  "projectId": "<project id>",
  "metadata": { "name": "My dApp", "description": "…", "url": "https://dapp.example", "icons": [] },
  "versions": [3],
  "requireDapp": "none",
  "storagePrefix": "wc3:"
}
```

Only `hosts`, `projectId` and `metadata` are required. The optional fields:
- **`versions`:** the protocol versions this client speaks.
- **`requireDapp`:** the wallet's policy, `none`, `monitored` or `sealed`.
- **`storagePrefix`:** the prefix on every storage key.
- **`audience`:** the Relay URL the JWT is signed for, when `hosts` points at a proxy.

**Free-form values.** Three values depend on the RPC method or event rather than on the protocol: a request's `params`, a response's `result` and an event's `data`. In the JSON facade they're ordinary JSON. In Rust, Swift and Kotlin they're JSON strings, as v2's Kotlin SDK passes request params. Everything else is typed everywhere.

### Rust

```rust
use sign_v3::{Namespace, Namespaces, ProposalParams, RpcRequest, SignClient, SignEvent};

let client = SignClient::init(config, storage, listener).await?;
client.start();

let eip155 = Namespace {
    chains: Some(vec!["eip155:1".into()]),
    accounts: None,
    methods: vec!["personal_sign".into()],
    events: vec!["chainChanged".into()],
};

// dApp: connect, show `uri` as a QR, wait for SignEvent::SessionConnect, then request
let params = ProposalParams {
    required_namespaces: Namespaces::from([("eip155".into(), eip155.clone())]),
    ..Default::default()
};
let connected = client.connect(params, None).await?;
let request = RpcRequest {
    method: "personal_sign".into(),
    params: r#"["0x68656c6c6f","0xAbc…"]"#.into(),
};
let signature = client.request(connected.topic, "eip155:1".into(), request, None).await?; // JSON, e.g. "\"0x…\""

// wallet: pair, approve, answer requests
let proposal = client.pair(uri).await?;
let mut approved = eip155;
approved.accounts = Some(vec!["eip155:1:0xAbc…".into()]);
let session = client
    .approve(proposal.id, Namespaces::from([("eip155".into(), approved)]), None)
    .await?;
// on SignEvent::SessionRequest { id, topic, chain_id, request }:
client.respond(topic, id, Some(r#""0xsignature""#.into()), None).await?;
```

### Swift and Kotlin

The uniffi bindings are typed. The app calls `SignClientFfi` and implements the generated `Storage` and `Listener` protocols:

```swift
final class AppStorage: Storage {
    func get(key: String) async throws -> String? { … }      // Keychain for keys under "secret/"
    func set(key: String, value: String) async throws { … }
    func remove(key: String) async throws { … }
    func keys(prefix: String) async throws -> [String] { … }
}
final class AppListener: Listener {
    func onEvent(event: SignEvent) {
        if case let .sessionRequest(id, topic, chainId, request) = event {
            // request.method, request.params (JSON string)
        }
    }
}

let client = try await SignClientFfi(config: config, storage: AppStorage(), listener: AppListener())
client.start()
let proposal = try await client.pair(uri: uri)
let session = try await client.approve(
    topic: proposal.id,
    namespaces: ["eip155": Namespace(chains: ["eip155:1"], accounts: ["eip155:1:0xAbc…"], methods: ["personal_sign"], events: ["chainChanged"])],
    own: nil
)
```

Kotlin has the same shape, with `suspend` functions, data classes and a `Storage` interface. `call(method:paramsJson:)` is also exported, for React Native bridges.

### Web (WASM)

```js
import init, { WasmSignClient } from "./pkg/sign_v3.js";

await init();
const storage = {
  get: async (k) => localStorage.getItem(k),
  set: async (k, v) => localStorage.setItem(k, v),
  remove: async (k) => localStorage.removeItem(k),
  keys: async (prefix) => Object.keys(localStorage).filter((k) => k.startsWith(prefix)),
};
const client = await WasmSignClient.create(JSON.stringify(config), storage, (json) => onEvent(JSON.parse(json)));
client.start();

const call = async (method, params) => JSON.parse(await client.call(method, JSON.stringify(params ?? {})));
const { uri, topic } = await call("connect", { params: { requiredNamespaces } });
```

A few rules for the storage object:
- **`keys(prefix)`** returns full keys, prefix included.
- **`localStorage` is for demos.** In production use IndexedDB, with keys under `secret/` encrypted.

A failed `call` rejects with an error JSON string; see [Errors](#errors).

In Node 20 and later, pass the module's bytes to `init` instead of letting it fetch them: `await init({ module_or_path: readFileSync("pkg/sign_v3_bg.wasm") })`.

### JSON facade: methods

`call(method, paramsJson)` resolves with the result as JSON. `null` means no result.

| Method | Params | Result |
|---|---|---|
| `connect` | `{ "params": ProposalParams, "intent"?: Intent }` | `{ "uri", "topic" }` |
| `pair` | `{ "uri" }` | `SessionProposal` |
| `approve` | `{ "topic", "namespaces": Namespaces, "own"?: "monitored" \| "sealed" }` | `Session` |
| `reject` | `{ "topic", "reason"? }` | `null` |
| `request` | `{ "topic", "chainId", "request": { "method", "params" }, "deadlineS"? }` | The wallet's result, as JSON |
| `respond` | `{ "topic", "id", "result"? }` or `{ "topic", "id", "error": { "code", "message" } }` | `null` |
| `emit` | `{ "topic", "chainId", "event": { "name", "data" } }` | `null` |
| `ping` | `{ "topic" }` | `null` |
| `update` | `{ "topic", "namespaces": Namespaces }`, wallet only | `null` |
| `extend` | `{ "topic", "expiry"? }`, wallet only, Unix seconds, at most 7 days ahead | `null` |
| `disconnect` | `{ "topic" }` | `null` |
| `getSessions` | none | `Session[]` |
| `clientId` | none | `"did:key:z…"` |

The shared shapes:
- **`Namespace`** is `{ "chains"?, "accounts"?, "methods", "events" }`, as in v2. **`Namespaces`** maps a namespace key such as `eip155` to a `Namespace`.
- **`ProposalParams`** is `{ "requiredNamespaces": Namespaces, "optionalNamespaces"?: Namespaces, "sessionProperties"?: { [key]: string } }`. That's v2's proposal params without the transport fields. The dApp's metadata is added from the config.
- **`Intent`** is `{ "own": "monitored" | "sealed", "peer": "none" | "monitored" | "sealed" }`, defaulting to `{ "own": "monitored", "peer": "none" }`.

Notes on the params:
- **`topic`** is the `connectionId`, a decimal string such as `"09178356979748733185"`. Treat it as opaque. On `approve` and `reject` it's the proposal's `id`.
- **`deadlineS`** defaults to 300 s and is capped at 7 days. 30 s past it, the call rejects with `message_expired`.
- **`respond` must answer a request this client received.** An unknown `id` rejects with `not_found`.

### Results

`SessionProposal`, returned by `pair` and carried by the `session_proposal` event:

```json
{
  "id": "09178356979748733185",
  "proposalId": 1759313100123000,
  "params": { "requiredNamespaces": { "eip155": { "chains": ["eip155:1"], "methods": ["personal_sign"], "events": ["chainChanged"] } } },
  "metadata": { "name": "My dApp", "description": "…", "url": "https://dapp.example", "icons": [] },
  "intent": { "own": "monitored", "peer": "none" },
  "versions": [3],
  "version": 3,
  "sealable": false,
  "expiresAt": 1759313400
}
```

- **`id`** is the `connectionId`. Pass it to `approve` or `reject`.
- **`proposalId`** is the JSON-RPC id, for apps that keep v2's numeric proposal ids.
- **`version`** is the version that will be used. `0` means no version is shared, and `pair` has already rejected the request.

`Session`, returned by `approve` and `getSessions`, and carried by `session_connect`:

```json
{
  "topic": "09178356979748733185",
  "role": "dapp",
  "version": 3,
  "namespaces": { "eip155": { "chains": ["eip155:1"], "accounts": ["eip155:1:0xAbc…"], "methods": ["personal_sign"], "events": ["chainChanged"] } },
  "peerMetadata": { "name": "My Wallet", "description": "…", "url": "https://wallet.example", "icons": [] },
  "dappMode": "monitored",
  "walletMode": "monitored",
  "expiry": 1759917600
}
```

### Events

The listener receives one object per event, tagged by `type`, with camelCase fields. On the web it arrives as a JSON string; Swift and Kotlin receive the `SignEvent` enum with the same fields.

| `type` | Fields | Who and when |
|---|---|---|
| `session_proposal` | `proposal` | Wallet, after `pair` |
| `session_connect` | `session` | dApp, when the wallet approved and the binding verified |
| `proposal_rejected` | `topic`, `reason` | dApp: `user_rejected`, `unsupported_version` or `incompatible_intent` |
| `proposal_expire` | `topic`, `reason` | dApp: `expired` with no answer in time, `replaced` by a newer `connect`, or `cursor_expired` |
| `proposal_failed` | `topic`, `reason` | dApp, when the wallet's binding failed verification; the client closed the connection |
| `session_request` | `id`, `topic`, `chainId`, `request` | The receiver; answer with `respond` |
| `session_response` | `topic`, `response`, `request` | A response with no pending call, such as after a restart; `request` is the stored request it answers |
| `session_event` | `id`, `topic`, `chainId`, `event` | The receiver; the client acknowledges it |
| `session_update` | `id`, `topic`, `namespaces` | dApp, after the wallet's `update` |
| `session_extend` | `id`, `topic`, `expiry` | dApp, after the wallet's `extend` |
| `session_ping` | `id`, `topic` | The receiver; the client acknowledges it |
| `session_delete` | `topic`, `reason`, `verified` | The other side closed (`verified` when its close signature checks out), the session expired, or `cursor_expired` |
| `message_expired` | `topic`, `seq`, `id` | Sender: no response by the deadline |
| `message_invalid` | `topic`, `reason` | A received message failed a check and was dropped |
| `cursor_expired` | none | Events were lost; every session was closed |
| `clock_skew` | `skewS` | The device clock is off by more than 5 minutes; the Relay's time is used |

### Errors

Errors carry a stable `code` and a `message`. The JSON facade rejects with `{ "code", "message" }` as a JSON string; Swift and Kotlin throw `SignError.Sdk(code, message)`.

| `code` | Meaning |
|---|---|
| `invalid_config` | `hosts` is empty |
| `unknown_method` | The JSON facade got a method it doesn't know |
| `invalid_params` | Arguments weren't valid I-JSON or were missing fields |
| `invalid_uri`, `unsupported_uri` | Not a `wc:` URI, or a v2 URI |
| `invalid_request` | The fetched request failed verification; show nothing to the user |
| `invalid_peer` | The peer's binding or approval failed verification |
| `unsupported_version`, `incompatible_intent` | No shared version, or the intents don't fit |
| `request_resolved` | The QR was already answered, or its window closed |
| `not_found` | Unknown session, proposal or request id, or a QR a newer `connect` replaced |
| `wallet_only` | `update` or `extend` called by the dApp |
| `connection_closed` | The session was closed |
| `message_expired` | No response by the deadline |
| `backpressure` | The receiver holds too many unacknowledged events; retry later |
| `rpc_error` | The peer answered with a JSON-RPC error. `message` is that error as JSON, `{ "code", "message" }` |
| `conflict`, `forbidden`, `unauthorized`, `rate_limited`, `relay_error`, `network`, `storage` | Relay, transport or storage failures |

## Test

```bash
cargo test -p sign-v3 --lib                                        # unit tests, offline
SIGN_V3_RELAY_HOSTS=https://<v3 relay> cargo test -p sign-v3 --features rustls   # everything
cargo +nightly fmt -p sign-v3 --check
cargo clippy -p sign-v3 --features uniffi,bindgen --all-targets -- -D warnings
cargo clippy -p sign-v3 --no-default-features --features wasm --target wasm32-unknown-unknown -- -D warnings
```

| Suite | Covers | Needs |
|---|---|---|
| Unit (`--lib`) | The cryptography test vectors and negative vectors, strict I-JSON, the URI, the JSON shapes of the public types | Nothing |
| `tests/e2e.rs` | Pairing; monitored and sealed round trips; a signed rejection; a newer `connect` replacing the pending one; the wallet's intent policy; wallet-only update and extend; a verified close; restoring from storage; requests answered and matched after a restart; tamper detection; refusing v2 URIs | `SIGN_V3_RELAY_HOSTS` |
| `tests/failover.rs` | A silent host is detected and both clients move to the other host | A Relay with debug routes and two hosts |
| `tests/interop.rs` | A Rust dApp with a TypeScript wallet, and the reverse | `SIGN_V3_TS_PEER_DIR`: a directory whose `scripts/ts-peer.ts` plays either side |

Without `SIGN_V3_RELAY_HOSTS` the integration tests skip. Some tests need a Relay with debug routes, used to check what the Relay saw, tamper with a message, or silence a host. Those tests skip themselves on a Relay without them, and so do the sealing tests on a Relay without sealing.

## Limits and open work

- **Not wired into packaging yet:**
  - `kotlin-ffi`, the Swift XCFramework and the release pipelines;
  - React Native bridges and the Flutter bindings.
- **The web host is left to the app:** IndexedDB storage, WebCrypto for secrets, lifecycle events.
- **One async mutex guards the client state,** rather than a dedicated actor task. HTTP never runs under it.
- **Some paths have no end-to-end test yet,** because they need a controllable Relay clock: losing events (`cursor_expired`) and the local expiries.

## WASM size

| Build | Brotli | Raw |
|---|---|---|
| Stable release, `wasm-opt -Oz` | 180 KB | — |
| Nightly size flags | 158 KB | 555 KB |

The nightly build uses `-Z build-std` with `optimize_for_size`, `-Cpanic=immediate-abort`, `-Zlocation-detail=none` and `-Zfmt-debug=none`. The release profile already uses `opt-level = "z"`, LTO and one codegen unit.

```bash
RUSTFLAGS="-Zunstable-options -Cpanic=immediate-abort -Zlocation-detail=none -Zfmt-debug=none" \
cargo +nightly build -p sign-v3 --release --target wasm32-unknown-unknown \
  --no-default-features --features wasm \
  -Z build-std=std,panic_abort -Z build-std-features=optimize_for_size
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/sign_v3.wasm
```

Where the bytes go, measured with [twiggy](https://github.com/rustwasm/twiggy):

| Source | Share |
|---|---|
| serde and serde_json | 30% |
| The crate's own logic | 30% |
| Static data | 8% |
| `core` and `alloc`, formatting included | 8% |
| Crypto | 7% |
| wasm-bindgen glue and futures | 6% |

Ways to shrink it further, not done:
- **Replace serde and serde_json** with a small JSON value and hand-written encoding around `protocol::ijson`.
- **Merge small async helpers** and move logic into synchronous functions.
- **Shorten error strings** and avoid `format!` on hot paths.

A realistic floor is about 90 to 110 KB brotli.
