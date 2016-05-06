extern crate bincode;
extern crate rustc_serialize;
extern crate rpsrtsrs;
extern crate docopt;

use std::net::TcpStream;
use std::ops::Deref;

use rpsrtsrs::state::{Game};
use rpsrtsrs::network::{Message};

use docopt::Docopt;

use bincode::SizeLimit;
use bincode::rustc_serialize::{decode_from, encode_into};
use bincode::rustc_serialize::DecodingResult;

static USAGE: &'static str = "
Usage: cli_client [-p PORT] [-i IP] [-r ID]

Options:
    -p PORT  The port to connect to [default: 8080].
    -i IP    The ipv4 address to connect to [default: 127.0.0.1].
    -r ID    Reconnect with the given ID
";

#[derive(RustcDecodable, Debug)]
struct Args {
    flag_p: u16,
    flag_i: String,
    flag_r: Option<u32>,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode())
                                       .unwrap_or_else(|e| e.exit());
    let host = args.flag_i;
    let port = args.flag_p;
    let reconnect = args.flag_r;

    println!("connecting to host: {:?}:{:?} reconnect? {:?}", host, port, reconnect);

    let mut stream = TcpStream::connect((host.deref(), port)).unwrap();

    match reconnect {
        Some(id) => {
            encode_into(&Message::ClientReconnect(id.into()), &mut stream, SizeLimit::Infinite).unwrap();
        },
        None => {
            encode_into(&Message::ClientHello, &mut stream, SizeLimit::Infinite).unwrap();
        }
    }
    let server_hello: DecodingResult<Message> = decode_from(&mut stream, SizeLimit::Infinite);
    println!("{:?}", server_hello);

    loop {
        let game_state: DecodingResult<Game> = decode_from(&mut stream, SizeLimit::Infinite);
        match game_state {
            Ok(game) => println!("{:?}", game),
            Err(e) => {
                println!("{:?}", e);
                return;
            }
        }
    }
}

