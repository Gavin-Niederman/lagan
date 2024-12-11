use std::{thread::sleep, time::Duration};

use lagan::{nt_types::PubSubOptions, prelude::*};
use log::{info, LevelFilter};
use pollster::FutureExt;
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

    let topic = client.topic("/sin");
    let topic_subscriber = topic.subscribe(ValueType::F64, "double", Default::default());
    let topic2 = client.topic("/iCanPublish");
    let topic_publisher = topic2.publish(ValueType::F64, "double", PubSubOptions::default());

    let entry = client.entry("/sinRecieved");

    async {
        for i in 0.. {
            info!("topic is of type: {:?}", topic.value_type());
            info!("topic exists? {:?}", topic.is_existant());
            if topic.is_existant() {
                let latest = topic_subscriber.value_f64().await;
                info!("latest update: {:?}", latest);
                entry.set_value_f64(latest.unwrap()).unwrap();
            }
            topic_publisher.set_value_f64(3.14 + i as f64).unwrap();
            sleep(Duration::from_millis(200));
        }
    }
    .block_on();
}
