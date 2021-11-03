# Tauri Plugin Highlander

A Tauri Plugin to ensure there can only be 1 instance of a tauri application running and emits the second instances arguments to the Javascript via an event listener.


The plugin was named **Highlander** based on the [Film/Series Franchise](https://en.wikipedia.org/wiki/Highlander_(franchise)) where the immortals seek the "quickening" to make them stronger. The immortals hold to the motto "There can only be one".

# Architecture

The main process (your tauri app) upon initiation of the Highlander plugin will search for existing processes by the same name.

If no existing process is found then it will create a gRPC Server that will listen on and ipv6 loopback with a random port (defaults to `[::1]:0`).

If an existing process is found then it will search the existing listening sockets that is associated to that existing process ID then open a gRPC client to that instance (using the ip/port found in the socket list). This will trigger the existing instance to emit an event to the tauri-app passing in the opening arguments of the newest instance.

```mermaid

graph LR
  subgraph Application
    MainProcess
  end
  subgraph AsyncThread
    gRPCServer
  end
  subgraph AsyncBlockedThread
    Broadcaster
  end

  MainProcess --> |On Init| FindExisting --> Existing{Found?}
  Existing --> |Yes| Broadcaster --> ExitProcess
  Existing --> |No| gRPCServer

```

This is an example connectivity between the Existing Instance and a New Instance being launched.

```mermaid

graph TD
  subgraph Existing
    Instance-1
    gRPC-Server
    WebApp
  end
  subgraph New
    Instance-2
    Broadcaster
    Exit
  end
  Instance-1 --> |"Listening on <br /> [::1]:60432" | gRPC-Server
  Instance-1 --> |"Emits JS Event<br/>with payload"| WebApp
  gRPC-Server --> |"Event Payload"| Instance-1
  Instance-2 --> |"Finds existing <br />process on [::1]:60432"| Broadcaster --> gRPC-Server
  Broadcaster --> |"After Broadcaster completes"| Exit

```

# Getting Started

The Highlander Builder was created to make the intitiation of the plugin easier and to allow for features later without a major refactor. The builder has a `default()` method that will setup the following variables

| Variable | Description | Value |
| -- | -- | -- |
| event | The event name to broadcast to the WebApp | `quickening` |
| label | The label of the window to broadcast the event | `main` |
| listener | The `SocketAddress` to listen to for the gRPC Opener server | `[::1]:0` |

Each variable can be set after initializing with `default()` by method chaning.

```rust
HighlanderBuilder::default()
    .event("yourEventName".to_string())
```