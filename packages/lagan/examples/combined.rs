use std::thread::sleep;

use lagan::{client::Client, server::Server, Instance, NetworkTablesVersion};
use log::{info, LevelFilter};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

fn main() {
    TermLogger::init(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Stdout,
        ColorChoice::Auto,
    )
    .unwrap();

    let client = Client::builder()
        .address("127.0.0.1:5810".parse().unwrap())
        .version(NetworkTablesVersion::V4)
        .build();

    let server = Server::builder()
        .persist_filename("networktables.json")
        .build();

    let foo = client.entry("/foo");

    loop {
        info!("{:?}", foo.value_f64());
        sleep(std::time::Duration::from_millis(200));
    }
}
