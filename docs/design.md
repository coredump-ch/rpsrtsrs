# Design

The gamestate is handled completly by the server.  The gamestate consists of an
array of players which consist of a ID and an array of units.

The clients send commands to the server and receive the current gamestate.

## Network protocol

### Connect

Hello from client! Server responds with an ID that was assigned to the client.

After that only Update Gamestate commands follow.

### Update Gamestate

The server sends the current gamestate to the client.

### Command

Client sends a move command to the server.

