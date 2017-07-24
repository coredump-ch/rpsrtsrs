extern crate bincode;
#[macro_use]
extern crate serde_derive;
extern crate rpsrtsrs;
extern crate docopt;

use std::net::TcpStream;
use std::ops::Deref;
use std::io::Write;
use std::{thread, time};

use rpsrtsrs::state::GameState;
use rpsrtsrs::network::{Command, Message};

use docopt::Docopt;

use bincode::{serialize_into, deserialize_from, Infinite};
use bincode::internal::Result;

static USAGE: &'static str = "
Usage: cli_client [-p PORT] [-i IP] [-r ID] (read|move <id> <x> <y>)

Options:
    -p PORT  The port to connect to [default: 8080].
    -i IP    The ipv4 address to connect to [default: 127.0.0.1].
    -r ID    Reconnect with the given ID
";

#[derive(Deserialize, Debug)]
struct Args {
    flag_p: u16,
    flag_i: String,
    flag_r: Option<u32>,

    cmd_read: bool,
    arg_id: Option<u32>,
    arg_x: Option<f64>,
    arg_y: Option<f64>,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.deserialize())
                                       .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);
    let host = args.flag_i;
    let port = args.flag_p;
    let reconnect = args.flag_r;
    let cmd_read = args.cmd_read;

    println!("connecting to host: {:?}:{:?} reconnect? {:?}", host, port, reconnect);

    let mut stream = TcpStream::connect((host.deref(), port)).unwrap();

    match reconnect {
        Some(id) => {
            serialize_into(&mut stream,
                           &Message::ClientReconnect(id.into()),
                           Infinite)
                .unwrap();
        }
        None => {
            serialize_into(&mut stream, &Message::ClientHello, Infinite).unwrap();
        }
    }
    let server_hello: Result<Message> = deserialize_from(&mut stream, Infinite);
    println!("{:?}", server_hello);

    if cmd_read {
        loop {
            let game_state: Result<GameState> = deserialize_from(&mut stream,
                                                                 Infinite);
            match game_state {
                Ok(game) => println!("{:?}", game),
                Err(e) => {
                    println!("{:?}", e);
                    return;
                }
            }
        }
    } else {
        let id = args.arg_id.expect("<id> missing");
        let x = args.arg_x.expect("<x> missing");
        let y = args.arg_y.expect("<y> missing");
        serialize_into(&mut stream,
                       &Message::Command(Command::Move(id.into(), [x, y])),
                       Infinite)
            .unwrap();
        stream.flush().unwrap();
    }

    thread::sleep(time::Duration::from_millis(100));
}
