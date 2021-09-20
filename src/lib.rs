#[macro_use]
extern crate serde_derive;

#[cfg(feature = "include_glutin")]
extern crate glutin_window;

#[macro_use]
extern crate log;

#[cfg(feature = "include_sdl2")]
extern crate sdl2_window;

pub mod client;
pub mod colors;
pub mod common;
pub mod network;
pub mod server;
pub mod shapes;
pub mod state;
