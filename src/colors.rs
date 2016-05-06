//! Color definitions.
//!
//! This module should contain all colors used in the game. Later on, we could
//! replace it with a customizeable version, where people could write themes
//! (e.g. in TOML or YAML format).
//!
//! Make sure to contain only semantic names (e.g. "primary" or "background",
//! not "red" or "green").

pub const BLACK: [f32; 4] = [0.0, 0.0,  0.0, 1.0];
pub const YELLOW:[f32; 4] = [1.0, 1.0,  0.22, 1.0];
pub const ORANGE:[f32; 4] = [1.0, 0.61, 0.22, 1.0];
