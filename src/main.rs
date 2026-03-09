use std::{io, thread, time::Duration};

use netcode_serde_rs::{
    client::{ClientEvent, ClientPeer},
    server::{ServerEvent, ServerPeer},
};

const SERVER_ADDR: &str = "127.0.0.10:5000";
const CLIENT_BIND: &str = "0.0.0.0:0";

#[derive(serde::Serialize, serde::Deserialize, Debug)]
enum Message {
    Ping,
    Pong,
}

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
    let mut server = ServerPeer::<Message>::new(SERVER_ADDR)?;

    loop {
        let Some(event) = server.poll()? else {
            continue;
        };

        match event {
            ServerEvent::NewConnection(peer_id) => {
                println!("[ SERVER ] Nouvelle connexion : {:?}", peer_id);
            }
            ServerEvent::Disconnection(peer_id) => {
                println!("[ SERVER ] Déconnexion : {:?}", peer_id);
            }
            ServerEvent::Data(id, data) => {
                println!("[ SERVER ] from {:?} : Data = {:?}", id, data);
            }
        }
    }
}

fn client() -> io::Result<()> {
    let mut client = ClientPeer::<Message>::new(CLIENT_BIND, SERVER_ADDR)?;

    client.connect(Duration::from_secs(5), Duration::from_secs(1))?;

    println!("[ CLIENT ] Connexion au serveur");

    loop {
        let Some(event) = client.poll()? else {
            continue;
        };

        match event {
            ClientEvent::Disconnect => {
                println!("[ CLIENT ] Déconnexion du serveur");
            }
            ClientEvent::Data(data) => {
                println!("[ CLIENT ] Data = {:?}", data);
            }
        }
    }
}
