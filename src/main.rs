use std::{io, thread, time::Duration};

use netcode::{
    client::{ClientEvent, ClientPeer},
    server::ServerPeer,
};

const SERVER_ADDR: &str = "127.0.0.1:5000";
const CLIENT_BIND: &str = "0.0.0.0:0";

fn main() {
    thread::scope(|s| {
        s.spawn(|| log_error("[ SERVER ]", server));
        s.spawn(|| log_error("[ CLIENT ]", client));
    });
}

fn log_error<F: Fn() -> io::Result<()>>(prefix: &str, f: F) {
    if let Err(e) = f() {
        eprintln!("{prefix} {e}");
    }
}

fn server() -> io::Result<()> {
    let mut server = ServerPeer::new(SERVER_ADDR)?;

    println!("[ SERVER ] < Connected");

    loop {
        let Some(event) = server.poll()? else {
            continue;
        };

        println!("[ SERVER ] < {:?}", &event);
    }
}

fn client() -> io::Result<()> {
    let mut client = ClientPeer::new(CLIENT_BIND, SERVER_ADDR)?;

    client.connect(Duration::from_secs(5), Duration::from_secs(1))?;

    println!("[ CLIENT ] < Connected");

    loop {
        let Some(event) = client.poll()? else {
            continue;
        };

        println!("[ CLIENT ] < {:?}", &event);

        if let ClientEvent::Disconnected = event {
            return Ok(());
        }
    }
}
