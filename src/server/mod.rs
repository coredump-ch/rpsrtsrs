use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{Mutex,Arc};
use std::thread;
use std::time::Duration;

use bincode::SizeLimit;
use bincode::rustc_serialize::{decode_from, encode};
use bincode::rustc_serialize::DecodingResult;

use state::{WorldState, Player, Unit};
use network::{Message, Command};

pub type SafeWorldState = Arc<Mutex<WorldState>>;

pub fn handle_client(mut stream: TcpStream, world: SafeWorldState) {

    // handle client hello
    let client_message: DecodingResult<Message> = decode_from(&mut stream, SizeLimit::Bounded(128));
    match client_message {
        Ok(message) => {
            match message {
                Message::ClientHello => {
                    // Get exclusive world access
                    let mut world_lock = world.lock().unwrap();

                    // Get next free ID. This assumes the players list is
                    // sorted ascending by ID.
                    let id = world_lock.game.players.last()
                                                    .map_or(0, |player| player.id.0 + 1);

                    // Create new player for the newly connected client
                    let mut player = Player::new(id);

                    // Create four initial units for the player
                    player.units.push(Unit::new_random([ 50,  50]));
                    player.units.push(Unit::new_random([ 50, 100]));
                    player.units.push(Unit::new_random([100,  50]));
                    player.units.push(Unit::new_random([100, 100]));

                    // Add player to the world
                    let player_id = player.id;
                    world_lock.game.players.push(player);

                    // Send ServerHello message
                    let encoded: Vec<u8> = encode(
                        &Message::ServerHello(player_id, world_lock.clone()),
                        SizeLimit::Infinite
                    ).unwrap();
                    stream.write(&encoded).unwrap();
                },
                Message::ClientReconnect(id) => {
                    // Get exclusive world access
                    let world_lock = world.lock().unwrap();

                    // Find player with specified id
                    match world_lock.game.players.iter().find(|player| player.id == id) {
                        Some(_) => {
                            println!("Found you :)");

                            // Send ServerHello message
                            let encoded: Vec<u8> = encode(
                                &Message::ServerHello(id, world_lock.clone()),
                                SizeLimit::Infinite
                            ).unwrap();
                            stream.write(&encoded).unwrap();
                        },
                        None => {
                            println!("Reconnect to id {} not possible", id);

                            // Send Error message
                            let encoded: Vec<u8> = encode(
                                &Message::Error,
                                SizeLimit::Infinite).unwrap();
                            stream.write(&encoded).unwrap();
                            return  // Don't enter game loop
                        }
                    }
                },
                _ => {
                    println!("Did not receive ClientHello: {:?}", message);
                    let encoded: Vec<u8> = encode(&Message::Error, SizeLimit::Infinite).unwrap();
                    stream.write(&encoded).unwrap();
                    return  // Don't enter game loop
                }
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return  // Don't enter game loop
        }
    }

    let mut command_stream = stream.try_clone().unwrap();
    let world_clone = world.clone();
    // Command receiver loop
    thread::spawn(move || {
        loop {
            let client_message: DecodingResult<Message> = decode_from(&mut command_stream, SizeLimit::Bounded(128));
            match client_message {
                Ok(message) => {
                    match message {
                        Message::Command(command) => {
                            let world_lock = world_clone.lock().unwrap();
                            handle_command(&mut command_stream, &world_lock, &command);
                        },
                        _ => {
                            println!("Did receive unexpected message: {:?}", message);
                            let encoded: Vec<u8> = encode(&Message::Error, SizeLimit::Infinite).unwrap();
                            command_stream.write(&encoded).unwrap();
                            return
                        },
                    }
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                    return;
                }
            };
        }
    });

    // GameState loop
    loop {
        let encoded: Vec<u8> = {
            let world_lock = world.lock().unwrap();
            encode(&world_lock.game, SizeLimit::Infinite).unwrap()
        };
        match stream.write(&encoded) {
            Err(e) => {
                println!("Error: {:?}", e);
                return;
            }
            _ => thread::sleep(Duration::from_millis(1000)),
        };
    }
}

pub fn handle_command(mut stream: &TcpStream, world: &WorldState, command: &Command) {
    println!("Did receive command {:?}", command);
    match command {
        &Command::Move(id, target) => println!("Move {} to {:?}!", id, target),
    }
}

pub fn update_world(world: SafeWorldState) {
    loop {
        {
            let mut world_lock = world.lock().unwrap();
            for player in world_lock.game.players.iter_mut() {
                for unit in player.units.iter_mut() {
                    unit.update(500);
                }
                println!("{:?}", player);
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
}
