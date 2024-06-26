#[macro_use]
extern crate serde_derive;

use std::ops::Deref;

use docopt::Docopt;

use rpsrtsrs::server::Server;

static USAGE: &str = "
Usage: server [-p PORT] [-i IP]

Options:
    -p PORT  The port to listen on [default: 8080].
    -i IP    The ipv4 address to listen on [default: 127.0.0.1].
    -r ID    Reconnect with the given ID
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_p: u16,
    flag_i: String,
}

fn main() {
    env_logger::init();
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    let host = args.flag_i;
    let port = args.flag_p;

    let server =
        Server::new((host.deref(), port), (800.0, 600.0)).expect("Could not initialize server");
    server.serve();
}
