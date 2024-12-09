use std::thread::sleep;

use lagan::prelude::*;
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
        .build();

    let server = Server::builder()
        .persist_filename("networktables.json")
        .build();

    let foo_server = server.entry("/data");
    foo_server.set_value_string("aa").unwrap();
    foo_server.set_flags(ValueFlags::PERSISTENT).unwrap();
    let foo = client.entry("/data");

    loop {
        info!("{:?}", foo.value_string());
        sleep(std::time::Duration::from_millis(200));
    }
}
