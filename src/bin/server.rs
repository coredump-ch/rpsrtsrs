extern crate bincode;
extern crate rustc_serialize;
extern crate rpsrtsrs;
extern crate docopt;

use std::net::{TcpListener, TcpStream};
use std::io::Write;
use std::ops::Deref;
use std::sync::{Mutex,Arc};
use std::thread;
use std::time::Duration;

use docopt::Docopt;

use rpsrtsrs::state::{World, Player, Unit};
use rpsrtsrs::network::{Message};

use bincode::SizeLimit;
use bincode::rustc_serialize::{decode_from, encode};
use bincode::rustc_serialize::DecodingResult;

type SafeWorldState = Arc<Mutex<World>>;

fn handle_client(mut stream: TcpStream, world: SafeWorldState) {

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

fn update_world(world: SafeWorldState) {
    loop {
        {
            let mut world_lock = world.lock().unwrap();
            for player in world_lock.game.players.iter_mut() {
                for unit in player.units.iter_mut() {
                    unit.update(500);
                    println!("{:?}", unit);
                }
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
}

static USAGE: &'static str = "
Usage: server [-p PORT] [-i IP]

Options:
    -p PORT  The port to listen on [default: 8080].
    -i IP    The ipv4 address to listen on [default: 127.0.0.1].
    -r ID    Reconnect with the given ID
";

#[derive(RustcDecodable, Debug)]
struct Args {
    flag_p: u16,
    flag_i: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode())
                                       .unwrap_or_else(|e| e.exit());
    let host = args.flag_i;
    let port = args.flag_p;

    let tcp_listener = TcpListener::bind((host.deref(),port)).unwrap();
    println!("Start server: {:?}", tcp_listener);

    let world = Arc::new(Mutex::new(World::new(800, 600)));
    let world_clone = world.clone();
    thread::spawn(move || {
        update_world(world_clone);
    });

    for stream in tcp_listener.incoming() {
        match stream {
            Ok(stream) => {
                let world_clone = world.clone();
                println!("Spawning thread...");
                thread::spawn(move || {
                    handle_client(stream, world_clone);
                });
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
    drop(tcp_listener);
}
