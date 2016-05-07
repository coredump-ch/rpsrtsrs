extern crate bincode;
extern crate rustc_serialize;
extern crate rpsrtsrs;
extern crate docopt;

use std::net::TcpListener;
use std::ops::Deref;
use std::sync::{Mutex, Arc};
use std::thread;

use docopt::Docopt;

use rpsrtsrs::state::WorldState;
use rpsrtsrs::server::{update_world, handle_client};

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

    let world = Arc::new(Mutex::new(WorldState::new(800, 600)));
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
