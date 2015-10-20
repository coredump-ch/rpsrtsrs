extern crate bincode;
extern crate rustc_serialize;
extern crate rpsrtsrs;
use std::net::TcpListener;
use std::io::Write;
use std::ops::Deref;

use rpsrtsrs::colors;
use rpsrtsrs::state::{Game, Player};

use bincode::SizeLimit;
use bincode::rustc_serialize::encode;

fn main() {
    let game = Game{
        players: vec![
            Player{
                id: 0,
                units: vec![],
            }]
    };
    let encoded: Vec<u8> = encode(&game, SizeLimit::Infinite).unwrap();

    let socket_addr = "127.0.0.1:8080".to_string();
    let tcp_listener = TcpListener::bind(socket_addr.deref()).unwrap();
    println!("Start server: {:?}", tcp_listener);
    for stream in tcp_listener.incoming() {
        let mut stream = stream.unwrap();
        stream.write(&encoded).unwrap();
    }
}
