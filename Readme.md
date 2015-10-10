[![Build Status](https://travis-ci.org/rnestler/rpsrtsrs.svg?branch=master)](https://travis-ci.org/rnestler/rpsrtsrs)

# rpsrtsrs

Rock-Paper-Scissors-Real-Time-Strategie written in Rust :wink:

![](docs/images/game-board-initial.png)

## Prequisites

* SDL 2 (`libsdl2-dev` on Ubuntu)
* FreeType (`libfreetype6-dev` on Ubuntu)

## Building

### Linux

Just run

    $ cargo build

To build with the default configuration. If you want to change the window
backend use any of the following:

    $ cargo build --no-default-features --features include_glutin
    $ cargo build --no-default-features --features include_glfw
    $ cargo build --no-default-features --features include_sdl2

### Windows

You will need to install [freetype](https://github.com/PistonDevelopers/freetype-sys#for-windows-users)

## Running

If everything built fine you can use cargo to run the client or the server:

    $ cargo run --bin client
    $ cargo run --bin server

## Ideas

See [ideas](ideas.md)
