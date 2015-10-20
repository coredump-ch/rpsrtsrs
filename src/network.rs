use state::{Game,World};

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub enum Command {
    /// Move command with unit ID and target
    Move(u32, [u64;2]),
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub enum Message {
    Error,
    ClientHello,
    ServerHello(u32, World),
    UpdateGamestate(Game),
    Command(Command),
}

