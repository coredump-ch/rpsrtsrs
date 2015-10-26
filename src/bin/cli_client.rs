extern crate bincode;
extern crate rustc_serialize;
extern crate rpsrtsrs;
use std::net::TcpStream;

use rpsrtsrs::state::{Game};
use rpsrtsrs::network::{Message};

use bincode::SizeLimit;
use bincode::rustc_serialize::{decode_from, encode_into};
use bincode::rustc_serialize::DecodingResult;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();

    encode_into(&Message::ClientHello, &mut stream, SizeLimit::Infinite).unwrap();
    let world: DecodingResult<Message> = decode_from(&mut stream, SizeLimit::Infinite);
    println!("{:?}", world);

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

