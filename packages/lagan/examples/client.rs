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
    println!("{:?}", client);

    let foo = client.entry("/foo");
    let sin = client.entry("/sin");

    sin.set_value_f64(0.0_f64.sin()).unwrap();
    sin.set_flags(ValueFlags::PERSISTENT).unwrap();

    foo.set_value_bool(true).unwrap();
    foo.set_flags(ValueFlags::PERSISTENT).unwrap();

    for i in 0.. {
        sin.set_value_f64((i as f64 / 2.0).sin()).unwrap();
        info!("{:?}", sin.value());
        foo.set_value_bool(i % 2 == 0).unwrap();
        info!("{:?}", foo.value());
        sleep(std::time::Duration::from_millis(200));
    }
}
