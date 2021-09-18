//! Data structures that can be shared between the server and the client.
//!
//! All these structures should be serializable, so that they can be
//! transferred from the server to the client over the network.
use std::collections::HashMap;
use std::convert::Into;
use std::fmt;

use common::Vec2;
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
    pub position: Vec2,

    /// Angle of the unit in radiant
    pub angle: f64,

    /// Direction and speed of the movement. The angle of the movement may be different then the
    /// angle of the unit. The unit is m per milli second.
    pub speed_vector: Vec2,

    /// Health of the unit
    pub health: u64,
}

impl Unit {
    pub fn new<T: Into<UnitId>>(id: T, position: Vec2) -> Unit {
        println!("Create unit at {:?}", position);
        Unit {
            id: id.into(),
            position,
            angle: 0.0f64,
            speed_vector: Vec2::new(0.0, 0.0),
            health: 100_000,
        }
    }

    pub fn update(&mut self, dt_ms: f64) {
        self.position += self.speed_vector * dt_ms;
    }

    pub fn shoot(&self, size: f64, speed: f64) -> Bullet {
        let position = Vec2::new(
            self.position.x + self.angle.cos() * size,
            self.position.y + self.angle.sin() * size,
        );
        let speed = Vec2::new(self.angle.cos() * speed, self.angle.sin() * speed);
        Bullet::new(position, speed)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Bullet {
    pub position: Vec2,
    pub speed_vector: Vec2,
}

impl Bullet {
    pub fn new(position: Vec2, speed: Vec2) -> Bullet {
        Bullet {
            position,
            speed_vector: speed,
        }
    }

    pub fn update(&mut self, dt_ms: f64) {
        self.position += self.speed_vector * dt_ms;
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
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            players: vec![],
            bullets: vec![],
        }
    }

    pub fn update_targets(&mut self, unit_targets: &HashMap<UnitId, Vec2>) {
        for player in self.players.iter_mut() {
            for unit in player.units.iter_mut() {
                if let Some(target) = unit_targets.get(&unit.id) {
                    let speed = 0.0001;
                    unit.speed_vector = (target - unit.position) * speed;
                } else {
                    unit.speed_vector = Vec2::new(0.0, 0.0);
                }
            }
        }

        // Check for collisions between all units
        for p1 in 0..self.players.len() {
            for u1 in 0..self.players[p1].units.len() {
                for u2 in u1 + 1..self.players[p1].units.len() {
                    if self.players[p1].units[u1]
                        .collision_detect(&self.players[p1].units[u2], UNIT_SIZE)
                    {
                        {
                            let (s1, s2) = {
                                let unit1 = &self.players[p1].units[u1];
                                let unit2 = &self.players[p1].units[u2];
                                unit1.collision_avoidance(unit2)
                            };
                            self.players[p1].units[u1].speed_vector = s1;
                            self.players[p1].units[u2].speed_vector = s2;
                        }
                    }
                }

                for p2 in p1 + 1..self.players.len() {
                    for u2 in 0..self.players[p2].units.len() {
                        if self.players[p1].units[u1]
                            .collision_detect(&self.players[p2].units[u2], UNIT_SIZE)
                        {
                            let (s1, s2) = {
                                let unit1 = &self.players[p1].units[u1];
                                let unit2 = &self.players[p2].units[u2];
                                unit1.collision_avoidance(unit2)
                            };
                            self.players[p1].units[u1].speed_vector = s1;
                            self.players[p2].units[u2].speed_vector = s2;
                        }
                    }
                }
            }
        }
    }

    pub fn update(&mut self, world: &WorldState, dt: f64) {
        for bullet in self.bullets.iter_mut() {
            bullet.update(dt);
        }

        // waiting for non-lexical lifetimes...
        {
            let bullets = &mut self.bullets;
            let players = &mut self.players;
            bullets.retain(|bullet| {
                // still inside world?
                if bullet.position[0] > world.x
                    || bullet.position[1] > world.y
                    || bullet.position[0] < 0.0
                    || bullet.position[1] < 0.0
                {
                    return false;
                }

                for player in players.iter_mut() {
                    for unit in player.units.iter_mut() {
                        if unit.health > 0 && unit.is_hit(UNIT_SIZE, bullet.position) {
                            if unit.health > 10000 {
                                unit.health -= 10000;
                            } else {
                                unit.health = 0;
                            }
                            println!("hit: {}", unit.health);
                            return false;
                        }
                    }
                }
                true
            });
        }

        // remove all units where health == 0
        for player in self.players.iter_mut() {
            player.units.retain(|unit| unit.health > 0);
            for unit in player.units.iter_mut() {
                unit.update(dt);
            }
        }
    }

    pub fn shoot(&mut self, id: UnitId) {
        for player in self.players.iter_mut() {
            for unit in player.units.iter() {
                if unit.id == id {
                    let bullet = unit.shoot(UNIT_SIZE, 0.1);
                    self.bullets.push(bullet);
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
        WorldState { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_update_stationary() {
        //! The position of a unit should not change on update when no speed
        //! vector is defined.
        let pos = Vec2::new(20.0, 10.0);
        let mut unit = Unit::new(1, pos);
        assert_eq!(unit.position, pos);
        unit.update(10.0);
        assert_eq!(unit.position, pos);
        unit.update(1000.0);
        assert_eq!(unit.position, pos);
    }

    #[test]
    fn test_unit_update_moving() {
        //! The position of a unit should not change on update when no speed
        //! vector is defined.
        let mut unit = Unit::new(1, Vec2::new(20.0, 10.0));
        unit.speed_vector = Vec2::new(1.0, 2.0);
        assert_eq!(unit.position, Vec2::new(20.0, 10.0));
        unit.update(1.0);
        assert_eq!(unit.position, Vec2::new(21.0, 12.0));
        unit.update(100.0);
        assert_eq!(unit.position, Vec2::new(121.0, 212.0));
    }

    #[test]
    fn test_bullet_update_stationary() {
        //! The position of a bullet should not change on update when a zero
        //! speed vector is defined.
        let pos = Vec2::new(20.0, 10.0);
        let speed = Vec2::new(0.0, 0.0);
        let mut bullet = Bullet::new(pos, speed);
        assert_eq!(bullet.position, pos);
        bullet.update(10.0);
        assert_eq!(bullet.position, pos);
        bullet.update(1000.0);
        assert_eq!(bullet.position, pos);
    }

    #[test]
    fn test_bullet_update_moving() {
        //! The position of a bullet should not change on update when no speed
        //! vector is defined.
        let mut bullet = Bullet::new(Vec2::new(20.0, 10.0), Vec2::new(1.0, 2.0));
        assert_eq!(bullet.position, Vec2::new(20.0, 10.0));
        bullet.update(1.0);
        assert_eq!(bullet.position, Vec2::new(21.0, 12.0));
        bullet.update(100.0);
        assert_eq!(bullet.position, Vec2::new(121.0, 212.0));
    }
}
