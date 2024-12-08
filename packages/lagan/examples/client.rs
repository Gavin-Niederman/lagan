use std::thread::sleep;

use lagan::{client::Client, NetworkTablesVersion};
use log::LevelFilter;
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
        .version(NetworkTablesVersion::V4)
        .address("127.0.0.1:5810".parse().unwrap())
        .build();
    println!("{:?}", client);

    loop {
        sleep(std::time::Duration::from_secs(1));
    }
}
