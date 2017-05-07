use std::io::Write;
use std::io::Result as IoResult;
use std::net::{TcpListener, TcpStream, SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::ops::RangeFrom;
use std::collections::HashMap;

use bincode::SizeLimit;
use bincode::rustc_serialize::{decode_from, encode};
use bincode::rustc_serialize::DecodingResult;

use state::{WorldState, Player, Unit, UnitId};
use network::{Message, Command};

/// A `Server` instance holds global server state.
pub struct Server {
    socket_addr: SocketAddr,
    world: Arc<Mutex<WorldState>>,
    /// Generator that returns sequential unit IDs
    unit_id_generator: Arc<Mutex<RangeFrom<u32>>>,
    /// Generator that returns sequential client IDs
    client_id_generator: Arc<Mutex<RangeFrom<u32>>>,

    /// Map with active unit move commands
    unit_targets: Arc<Mutex<HashMap<UnitId, [f64; 2]>>>,
}

impl Server {
    pub fn new<T: ToSocketAddrs>(addr: T,
                                world_size: (u64, u64))
                                -> IoResult<Server> {
        let addr = try!(addr.to_socket_addrs()).next().unwrap();
        let world = Arc::new(Mutex::new(WorldState::new(world_size.0, world_size.1)));
        Ok(Server {
            socket_addr: addr,
            world: world,
            client_id_generator: Arc::new(Mutex::new(0..)),
            unit_id_generator: Arc::new(Mutex::new(0..)),
            unit_targets: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn serve(&self) {
        let tcp_listener = TcpListener::bind(self.socket_addr).unwrap();
        println!("Start server: {:?}", tcp_listener);

        let world_clone = self.world.clone();
        let unit_targets_clone = self.unit_targets.clone();
        thread::spawn(move || {
            update_world(world_clone, unit_targets_clone);
        });

        for stream in tcp_listener.incoming() {
            match stream {
                Ok(stream) => {
                    let world_clone = self.world.clone();
                    let client_id_generator_clone = self.client_id_generator.clone();
                    let unit_id_generator_clone = self.unit_id_generator.clone();
                    let unit_targets = self.unit_targets.clone();
                    println!("Spawning thread...");
                    thread::spawn(move || {
                        handle_client(stream, world_clone,
                                      client_id_generator_clone, unit_id_generator_clone, unit_targets);
                    });
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        }
    }
}

pub type SafeWorldState = Arc<Mutex<WorldState>>;
pub type SafeUnitTargets = Arc<Mutex<HashMap<UnitId, [f64; 2]>>>;

pub fn handle_client(mut stream: TcpStream,
                     world: SafeWorldState,
                     client_id_generator: Arc<Mutex<RangeFrom<u32>>>,
                     unit_id_generator: Arc<Mutex<RangeFrom<u32>>>,
                     unit_targets: SafeUnitTargets) {

    // handle client hello
    let client_message: DecodingResult<Message> = decode_from(&mut stream, SizeLimit::Bounded(128));
    match client_message {
        Ok(message) => {
            match message {
                Message::ClientHello => {
                    // Get exclusive world access
                    let mut world_lock = world.lock().unwrap();

                    // Create new player for the newly connected client
                    let client_id = client_id_generator
                        .lock().expect("Could not lock client_id_generator mutex")
                        .next().expect("No more client IDs available!");
                    let mut player = Player::new(client_id);

                    // Create four initial units for the player
                    let coords = [
                        [50.0f64, 50.0f64], [50.0f64, 100.0f64], [100.0f64, 50.0f64], [100.0f64, 100.0f64],
                    ];
                    for coord in coords.iter() {
                        let unit_id = unit_id_generator
                            .lock().expect("Could not lock unit_id_generator mutex")
                            .next().expect("No more unit IDs available!");
                        player.units.push(Unit::new(unit_id, *coord));
                    }

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
    let unit_targets_clone = unit_targets.clone();
    // Command receiver loop
    thread::spawn(move || {
        loop {
            let client_message: DecodingResult<Message> = decode_from(&mut command_stream, SizeLimit::Bounded(128));
            match client_message {
                Ok(message) => {
                    match message {
                        Message::Command(command) => {
                            let mut world_lock = world_clone.lock().unwrap();
                            let mut unit_targets_lock = unit_targets_clone.lock().unwrap();
                            handle_command(&mut world_lock, &mut unit_targets_lock, &command);
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
            _ => thread::sleep(Duration::from_millis(10)),
        };
    }
}

pub fn handle_command(world: &mut WorldState, unit_targets: &mut HashMap<UnitId, [f64; 2]>, command: &Command) {
    println!("Did receive command {:?}", command);
    match command {
        &Command::Move(id, target) => {
            for player in world.game.players.iter_mut() {
                for unit in player.units.iter_mut() {
                    if unit.id == id {
                        println!("Found it :)");
                        unit_targets.insert(id, target);
                    }
                }
            }
            println!("Move {} to {:?}!", id, target);
        }
    }
}

pub fn update_world(world: SafeWorldState, unit_targets: SafeUnitTargets) {
    loop {
        {
            let mut world_lock = world.lock().unwrap();
            let mut unit_targets = unit_targets.lock().unwrap();
            for player in world_lock.game.players.iter_mut() {
                for unit in player.units.iter_mut() {
                    if let Some(target) = unit_targets.get(&unit.id) {
                        let speed = 0.0001;
                        unit.speed_vector = [(target[0]-unit.position[0])*speed, (target[1]-unit.position[1])*speed];
                    } else {
                        unit.speed_vector = [0.0,0.0];
                    }
                    unit.update(5);
                }
                //println!("{:?}", player);
            }
        }
        thread::sleep(Duration::from_millis(5));
    }
}
