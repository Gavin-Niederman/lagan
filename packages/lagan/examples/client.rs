use std::thread::sleep;

use lagan::{client::Client, Instance};
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
    println!("{:?}", client);

    let foo = client.entry("/foo");

    loop {
        info!("{:?}", foo.value());
        sleep(std::time::Duration::from_millis(200));
    }
}
