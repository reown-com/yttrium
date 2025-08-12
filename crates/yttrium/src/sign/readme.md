```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> MaybeReconnect: online()
    Idle --> ConnectRequest: request_rx
    MaybeReconnect --> ConnectSubscribe: sessions > 0
    MaybeReconnect --> Idle: sessions = 0
    ConnectSubscribe --> AwaitingSubscribeResponse: connected
    ConnectSubscribe --> Backoff: error/timeout
    Backoff --> MaybeReconnect: sleeped
    Backoff --> ConnectRequest: request_rx
    Connected --> MaybeReconnect: disconnected
    Connected --> AwaitingRequestResponse: request_rx
    AwaitingSubscribeResponse --> Connected: response received
    AwaitingSubscribeResponse --> Poisoned: auth error
    AwaitingConnectRequestResponse --> Poisoned: auth error
    ConnectRequest --> AwaitingConnectRequestResponse: connected
    AwaitingConnectRequestResponse --> Connected: response received
    AwaitingConnectRequestResponse --> MaybeReconnect: error/timeout
    Poisoned --> Poisoned: request_rx
    AwaitingSubscribeResponse --> Backoff: error/timeout
    ConnectRequest --> Idle: error/timeout
    AwaitingRequestResponse --> ConnectRetryRequest: error/timeout
    AwaitingRequestResponse --> Connected: response received
    ConnectRetryRequest --> MaybeReconnect: error/timeout
    AwaitingConnectRetryRequestResponse --> MaybeReconnect: error/timeout
    AwaitingConnectRetryRequestResponse --> Connected: response received
    ConnectRetryRequest --> AwaitingConnectRetryRequestResponse: connected
```
