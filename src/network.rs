use state::{GameState, WorldState, UnitId, ClientId};

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub enum Command {
    /// Move command with unit ID and target
    Move(UnitId, [u64;2]),
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub enum Message {
    Error,
    ClientHello,
    ClientReconnect(ClientId),
    ServerHello(ClientId, WorldState),
    UpdateGamestate(GameState),
    Command(Command),
}
