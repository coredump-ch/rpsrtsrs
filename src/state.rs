//! Data structures that can be shared between the server and the client.
//!
//! All these structures should be serializable, so that they can be
//! transferred from the server to the client over the network.
use std::convert::Into;
use std::fmt;
use std::collections::HashMap;


/// A unit identifier.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub struct UnitId(pub u32);

impl Into<UnitId> for u32 {
    fn into(self) -> UnitId {
        UnitId(self)
    }
}

impl fmt::Display for UnitId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

/// A client/player identifier.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone)]
pub struct ClientId(pub u32);

impl Into<ClientId> for u32 {
    fn into(self) -> ClientId {
        ClientId(self)
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}


/// The state of a single unit
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Unit {
    /// The unit identifier
    pub id: UnitId,

    /// X/Y position in the world in m
    pub position: [f64; 2],

    /// Angle of the unit in radiant
    pub angle: f64,

    /// Direction and speed of the movement. The angle of the movement may be different then the
    /// angle of the unit. The unit is m per milli second.
    pub speed_vector: [f64; 2],

    /// Health of the unit
    pub health: u64,
}

impl Unit {
    pub fn new<T: Into<UnitId>>(id: T, position: [f64; 2]) -> Unit {
        println!("Create unit at {:?}", position);
        Unit {
            id: id.into(),
            position: position,
            angle: 0.0f64,
            speed_vector: [0.0f64, 0.0f64],
            health: 100_0000,
        }
    }

    pub fn update(&mut self, dt_ms: f64) {
        self.position[0] += self.speed_vector[0] * dt_ms;
        self.position[1] += self.speed_vector[1] * dt_ms;
    }
}


/// A player has an ID and consists of 0..N `Unit`s
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Player {
    pub id: ClientId,
    pub units: Vec<Unit>,
}

impl Player {
    pub fn new<T: Into<ClientId>>(id: T) -> Player {
        Player {
            id: id.into(),
            units: vec![],
        }
    }
}

/// Data related to the current game.
///
/// This needs to be transferred to the client every time the game state
/// changes.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct GameState {
    /// List of players
    pub players: Vec<Player>,
}

impl GameState {
    pub fn new() -> GameState {
        GameState{ players: vec![] }
    }

    pub fn update_targets(&mut self, unit_targets: &HashMap<UnitId, [f64; 2]>) {
        for player in self.players.iter_mut() {
            for unit in player.units.iter_mut() {
                if let Some(target) = unit_targets.get(&unit.id) {
                    let speed = 0.0001;
                    unit.speed_vector = [(target[0]-unit.position[0])*speed, (target[1]-unit.position[1])*speed];
                } else {
                    unit.speed_vector = [0.0,0.0];
                }
            }
        }
    }

    pub fn update(&mut self, dt: f64) {
        for player in self.players.iter_mut() {
            for unit in player.units.iter_mut() {
                unit.update(dt);
            }
        }
    }
}

/// Data related to the entire world, like width and height.
///
/// This needs to be transferred to the client only once, on connecting.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WorldState {
    /// Width of the world in m
    pub x: f64,
    /// Height of the world in m
    pub y: f64,
}

impl WorldState {
    pub fn new(x: f64, y: f64) -> WorldState {
        WorldState {
            x: x,
            y: y,
        }
    }
}
