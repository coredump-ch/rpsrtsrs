/// The state of a single unit
#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct Unit {
    pub position: [u64; 2],
    pub angle: u64,
    pub speed_vector: [u64; 2],
    pub health: u64,
}

/// A player has a color and consists of N Unit's
#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct Player {
    pub color: [f32; 4],
    pub units: Vec<Unit>,
}

/// The whole game consists of N players
#[derive(RustcEncodable, RustcDecodable, PartialEq)]
pub struct Game {
    pub players: Vec<Player>,
}

