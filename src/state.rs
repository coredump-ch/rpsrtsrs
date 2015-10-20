/// The state of a single unit
#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub struct Unit {
    pub position: [u64; 2],
    pub angle: u64,
    pub speed_vector: [u64; 2],
    pub health: u64,
}

/// A player has an ID and consists of 0..N `Unit`s
#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub struct Player {
    pub id: u32,
    pub units: Vec<Unit>,
}

impl Player {
    pub fn new(id: u32) -> Player {
        Player{
            id: id,
            units: vec![],
        }
    }
}

/// The whole game consists of N players
#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub struct Game {
    pub players: Vec<Player>,
}

