# Design

The gamestate is handled completly by the server.  The gamestate consists of an
array of players which consist of a ID and an array of units.

The clients send commands to the server and receive the current gamestate.

## Network protocol

### State machine

This is the state machine on the Server:

    +-------+   +-----------+   +----------------------------+
    |*start*+--->ClientHello+--->ServerHello(ClientId, World)+-+
    +-------+   +-----------+   +----------------------------+ |
                                                               |
                +-----------+                                  |
                |*connected*<----------------------------------+--+-+
                ++-+--------+                                     | |
                 | |                                              | |
                 | | +----------------+   +---------------------+ | |
                 | +->Command(Command)+--->UpdateGamestate(Game)+-+ |
                 |   +----------------+   +---------------------+   |
                 |                                                  |
                 |   +-------------------------+                    |
                 +--->ClientReconnect(ClientId)+--------------------+
                     +-------------------------+

- Initially, the server waits for a `ClientHello` message.
- It responds with a `ServerHello` message that contains the client ID that can
  be used by the client for reconnecting with a `ClientReconnect` message when
  the connection was lost.
- Then the server enters a loop and waits for a `Command` from the client. When
  such a command results in a world change, the world is sent back to the client
  as an `UpdateGamestate` message.

### Connect

Hello from client! Server responds with an ID that was assigned to the client.

After that only Update Gamestate commands follow.

### Update Gamestate

The server sends the current gamestate to the client.

### Command

Client sends a move command to the server.

### References

Here are some interesting links about networking in games:

 - http://gafferongames.com/networking-for-game-programmers/what-every-programmer-needs-to-know-about-game-networking/

