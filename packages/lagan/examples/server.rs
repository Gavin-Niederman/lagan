use lagan::server::Server;

fn main() {
    let server = Server::builder()
        .persist_filename("networktables.json")
        .build();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
