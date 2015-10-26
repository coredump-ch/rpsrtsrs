/// The state of a single unit
#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug, Clone)]
pub struct Unit {
    /// X/Y position in the world in mm
    pub position: [u64; 2],
    /// Angle of the unit in milli degrees
    pub angle: u32,
    /// Direction and speed of the movement. The angle of the movement may be different then the
    /// angle of the unit. The unit is mm per milli second.
    pub speed_vector: [u64; 2],
    /// Health of the unit
    pub health: u64,
}

impl Unit {
    pub fn new(position: [u64;2]) -> Unit {
        println!("Create unit at {:?}", position);
        Unit {
            position: position,
            angle: 0,
            speed_vector: [0,0],
            health: 100_0000,
        }
    }
    pub fn update(&mut self, dt_ms: u64) {
        self.position[0] += self.speed_vector[0]*dt_ms;
        self.position[1] += self.speed_vector[1]*dt_ms;
    }
}


/// A player has an ID and consists of 0..N `Unit`s
#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug, Clone)]
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
#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug, Clone)]
pub struct Game {
    pub players: Vec<Player>,
}

/// The world consists of an x and y size
#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug, Clone)]
pub struct World {
    /// Width of the world in mm
    pub x: u64,
    /// Height of the world in mm
    pub y: u64,
    pub game: Game,
}

impl World {
    pub fn new(x: u64, y: u64) -> World {
        World {
            x: x,
            y: y,
            game: Game{
                players: vec![],
            }
        }
    }
}

