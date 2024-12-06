use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread::sleep,
};

use lagan::{Client, NetworkTablesVersion};

fn main() {
    let client = Client::new(
        NetworkTablesVersion::V4,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5810),
        Some("yo"),
    );
    println!("{:?}", client);

    loop {
        sleep(std::time::Duration::from_secs(1));
    }
}
