extern crate bincode;
extern crate rustc_serialize;
extern crate rpsrtsrs;
use std::net::TcpStream;

use rpsrtsrs::state::{Game};

use bincode::SizeLimit;
use bincode::rustc_serialize::decode_from;
use bincode::rustc_serialize::DecodingResult;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    let game_state: DecodingResult<Game> = decode_from(&mut stream, SizeLimit::Infinite);
    println!("{:?}", game_state);
}

