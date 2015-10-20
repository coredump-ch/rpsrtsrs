extern crate bincode;
extern crate rustc_serialize;
extern crate rpsrtsrs;
use std::net::TcpListener;
use std::io::Write;
use std::ops::Deref;
use std::sync::{Mutex,Arc};
use std::thread;

use rpsrtsrs::state::{Game, Player};

use bincode::SizeLimit;
use bincode::rustc_serialize::encode;

fn main() {
    let game = Arc::new(Mutex::new(Game{
        players: vec![],
    }));

    let socket_addr = "127.0.0.1:8080".to_string();
    let tcp_listener = TcpListener::bind(socket_addr.deref()).unwrap();
    println!("Start server: {:?}", tcp_listener);

    for stream in tcp_listener.incoming() {
        let game_clone = game.clone();
        thread::spawn(move || {
            let mut stream = stream.unwrap();
            let mut game_lock = game_clone.lock().unwrap();
            let id = game_lock.players.last().map_or(0, |player| player.id+1);

            // create new player for the newly connected client
            game_lock.players.push(Player::new(id));

            let encoded: Vec<u8> = encode(game_lock.deref(), SizeLimit::Infinite).unwrap();
            stream.write(&encoded).unwrap();
        });
    }
}
