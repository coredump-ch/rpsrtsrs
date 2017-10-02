//! Color definitions.
//!
//! This module should contain all colors used in the game. Later on, we could
//! replace it with a customizeable version, where people could write themes
//! (e.g. in TOML or YAML format).
//!
//! Make sure to contain only semantic names (e.g. "primary" or "background",
//! not "red" or "green").

pub const BLACK: [f32; 4] = [0.0, 0.0,  0.0, 1.0];
pub const WHITE: [f32; 4] = [1.0, 1.0,  1.0, 1.0];
pub const YELLOW:[f32; 4] = [1.0, 1.0,  0.22, 1.0];
pub const ORANGE:[f32; 4] = [1.0, 0.61, 0.22, 1.0];
pub const RED:[f32; 4] = [1.0, 0.22, 0.22, 1.0];
pub const LIGHT_BLUE:[f32; 4] = [0.22, 0.22, 1.0, 1.0];
pub const BLUE:[f32; 4] = [0.0, 0.0, 1.0, 1.0];

pub struct Player {
    pub primary: [f32; 4],
    pub secondary: [f32; 4],
}

pub const PLAYERS: [Player; 3] = [
    Player{ primary: ORANGE, secondary: YELLOW },
    Player{ primary: RED, secondary: ORANGE },
    Player{ primary: BLUE, secondary: LIGHT_BLUE },
];
