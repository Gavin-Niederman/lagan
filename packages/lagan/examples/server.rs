use lagan::server::Server;
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

    let server = Server::builder()
        .persist_filename("networktables.json")
        .build();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
