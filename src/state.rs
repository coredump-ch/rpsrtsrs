//! Data structures that can be shared between the server and the client.
//!
//! All these structures should be serializable, so that they can be
//! transferred from the server to the client over the network.
use std::convert::Into;
use std::fmt;
use std::collections::HashMap;
use std::mem;

use shapes::Shape;


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

pub const UNIT_SIZE: f64 = 50.0;

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
            health: 100_000,
        }
    }

    pub fn update(&mut self, dt_ms: f64) {
        self.position[0] += self.speed_vector[0] * dt_ms;
        self.position[1] += self.speed_vector[1] * dt_ms;
    }

    pub fn shoot(&self, size: f64, speed: f64) -> (Bullet, Laserbeam, Laserbeam) {
        let position = [
            self.position[0] + self.angle.cos() * size,
            self.position[1] + self.angle.sin() * size,
        ];
        let speed = [
            self.angle.cos() * speed,
            self.angle.sin() * speed,
        ];
        (
            Bullet::new(position, speed),
            Laserbeam::new(position, speed),
            Laserbeam::new(position, speed),
        )
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Bullet {
    pub position: [f64; 2],
    pub speed_vector: [f64; 2],
}

impl Bullet {
    pub fn new(position: [f64; 2], speed: [f64; 2]) -> Bullet {
        Bullet {
            position: position,
            speed_vector: speed,
        }
    }

    pub fn update(&mut self, dt_ms: f64) {
        self.position[0] += self.speed_vector[0] * dt_ms;
        self.position[1] += self.speed_vector[1] * dt_ms;
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Laserbeam {
    pub position_start: [f64; 2],
    pub position_head: [f64; 2],
    pub speed_vector: [f64; 2],
    pub length: f64,
}

impl Laserbeam {
    pub fn new(position_start: [f64; 2], speed_vector: [f64; 2]) -> Self {
        Laserbeam {
            position_start: position_start,
            position_head: position_start,
            speed_vector: speed_vector,
            length: 0.0,
        }
    }

    // TODO: This should be a trait
    pub fn update(&mut self, dt_ms: f64) {
        let length_per_ms = 1.0;
        self.length = self.length + (dt_ms * length_per_ms);
        self.position_head[0] = self.position_start[0] + self.speed_vector[0] * self.length;
        self.position_head[1] = self.position_start[1] + self.speed_vector[1] * self.length;
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
    pub bullets: Vec<Bullet>,
    pub laserbeams: Vec<Laserbeam>,
}

impl GameState {
    pub fn new() -> GameState {
        GameState{
            players: vec![],
            bullets: vec![],
            laserbeams: vec![],
        }
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

    pub fn update(&mut self, world: &WorldState, dt: f64) {
        let mut bullets = mem::replace(&mut self.bullets, vec![]);
        'bullet: for bullet in bullets.iter_mut() {
            bullet.update(dt);
            // still inside world?
            if bullet.position[0] > world.x || bullet.position[1] > world.y ||
                bullet.position[0] < 0.0 || bullet.position[1] < 0.0 {
                    continue;
            }

            for player in self.players.iter_mut() {
                for unit in player.units.iter_mut() {
                    if unit.health > 0 && unit.is_hit(UNIT_SIZE, bullet.position) {
                        if unit.health > 10000 {
                            unit.health -= 10000;
                        } else {
                            unit.health = 0;
                        }
                        println!("hit: {}", unit.health);
                        continue 'bullet;
                    }
                }
            }
            self.bullets.push(bullet.clone());
        }
        let mut laserbeams = mem::replace(&mut self.laserbeams, vec![]);
        for beam in laserbeams.iter_mut() {
            beam.update(dt);
            self.laserbeams.push(beam.clone());
            /*if beam.length > 9999.0 { // magic value do not change!!!!!!!!
                self.laserbeams.push(beam.clone());
            }*/
        }
        for player in self.players.iter_mut() {
            let mut units = mem::replace(&mut player.units, vec![]);
            for unit in units.iter_mut() {
                if unit.health > 0 {
                    unit.update(dt);
                    player.units.push(unit.clone());
                } else {
                    println!("killed unit!");
                }
            }
        }
    }

    pub fn shoot(&mut self, id: UnitId) {
        for player in self.players.iter_mut() {
            for unit in player.units.iter() {
                if unit.id == id {
                    let (bullet, beam_left, beam_right) = unit.shoot(UNIT_SIZE, 0.1);
                    self.bullets.push(bullet);
                    println!("shoot two beams");
                    self.laserbeams.push(beam_left);
                    self.laserbeams.push(beam_right);
                }
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
