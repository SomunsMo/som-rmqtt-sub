use std::collections::HashMap;
use std::fs::read_to_string;
use std::process::Command;
use std::time::Duration;

use rumqttc::{Client, Event, Incoming, MqttOptions, QoS, Transport};
use serde::Deserialize;

// --------------------------
// JSON5 配置结构
// --------------------------
#[derive(Debug, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub commands: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub use_tls: bool,
    pub username: String,
    pub password: String,
    pub client_id: String,
    pub topic: String,
}

fn main() {
    // 读取配置
    let config_str = read_to_string("config.json5").expect("Could not find config.json5 file");
    let config: Config = json5::from_str(&config_str).expect("Invalid configuration format");

    // MQTT 连接
    let mut mqtt_opts =
        MqttOptions::new(&config.mqtt.client_id, &config.mqtt.host, config.mqtt.port);
    mqtt_opts.set_credentials(&config.mqtt.username, &config.mqtt.password);
    mqtt_opts.set_keep_alive(Duration::from_secs(30));

    // -------------------------------------------------------
    // 如果配置开启了 TLS，则加载加密传输
    // -------------------------------------------------------
    if config.mqtt.use_tls {
        println!("SSL/TLS Enabled (MQTTS)");
        // 使用系统默认配置（信任系统根证书）
        mqtt_opts.set_transport(Transport::tls_with_default_config());
    } else {
        println!("SSL/TLS Disabled (MQTT)");
    }
    println!("---");
    // -------------------------------------------------------

    let (client, mut connection) = Client::new(mqtt_opts, 10);
    client
        // QOS（巴法云禁止Qos2，需要注意）
        .subscribe(&config.mqtt.topic, QoS::AtMostOnce)
        .unwrap();

    println!("Connect to MQTT...");

    // 循环监听
    loop {
        match connection.recv() {
            Ok(event) => {
                // 先匹配 Event → 再取 Incoming → 再取 Publish
                match event {
                    Ok(Event::Incoming(incoming)) => {
                        match incoming {
                            Incoming::ConnAck(_ack) => {
                                println!("Connected successfully | Topic: {}", config.mqtt.topic);
                                println!("---");
                            }
                            Incoming::Publish(p) => {
                                let payload = String::from_utf8_lossy(&p.payload);
                                let msg = payload.trim();
                                println!("\nReceived:{}", msg);

                                // 执行命令
                                if let Some(cmd) = config.commands.get(msg) {
                                    println!("Execute:{}", cmd);
                                    let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
                                }
                            }
                            _ => {
                                // println!("Received unexpected packet: {:?}", incoming);
                            }
                        }
                    }
                    _ => {}
                }
            }

            Err(e) => {
                eprintln!("Error:{:?}\n Reconnecting in 2S", e);
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }
}
