
/// The state of a single unit
pub struct Unit {
    position: [u64; 2],
    angle: u64,
    speed_vector: [u64; 2],
    health: u64,
}

/// A player has a color and consists of N Unit's
pub struct Player {
    color: [f32; 4],
    units: Vec<Unit>,
}

/// The whole game consists of N players
pub struct Game {
    players: Vec<Player>,
}

