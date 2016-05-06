use state::{Game, World, UnitId, ClientId};

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
    ServerHello(ClientId, World),
    UpdateGamestate(Game),
    Command(Command),
}
