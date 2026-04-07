use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::{Command, exit};
use std::time::Duration;

use rumqttc::{Client, ConnectionError, Event, Incoming, MqttOptions, QoS, Transport};
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

/// 解析配置文件路径
fn parse_config_path() -> PathBuf {
    let args: Vec<String> = env::args().collect();

    for i in 0..args.len() {
        // 支持多种参数格式：-config, --config, /config
        if (args[i] == "-config" || args[i] == "--config" || args[i] == "/config")
            && i + 1 < args.len()
        {
            return PathBuf::from(&args[i + 1]);
        }
    }

    // 默认从当前目录读取
    let path = PathBuf::from("config.json5");
    path
}

/// 处理连接错误
fn handler_conn_err(e: ConnectionError) {
    match e {
        ConnectionError::ConnectionRefused(_) => {
            // 认证相关错误，直接退出
            eprintln!("Connection refused.\nExiting...");
            exit(1);
        }
        _ => {}
    }
}

fn main() {
    // 打印程序启动信息，防止log与上次运行的日志视觉粘连
    println!();
    println!("======================================================");
    println!("MQTT Client Starting...");

    // 读取配置
    let config_path = parse_config_path();
    println!("Config path: {:?}", config_path);

    let config_str = read_to_string(&config_path).unwrap_or_else(|_| {
        eprintln!("Could not find config file at {:?}", config_path);
        exit(1);
    });
    let config: Config = json5::from_str(&config_str).unwrap_or_else(|_| {
        eprintln!("Invalid configuration format in {:?}.", config_path);
        exit(1);
    });

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

    println!(
        "Attempting to connect to {}:{}",
        config.mqtt.host, config.mqtt.port
    );

    // 超时重试时长
    let sleep_duration = Duration::from_secs(5);

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
                            Incoming::Disconnect => {
                                eprintln!("Disconnected from MQTT broker");
                            }
                            _ => {
                                // println!("Received unexpected packet: {:?}", incoming);
                            }
                        }
                    }
                    Ok(Event::Outgoing(_outgoing)) => {
                        // 处理 outgoing 事件（可选）
                        // println!("Outgoing event: {:?}", _outgoing);
                    }
                    Err(e) => {
                        eprintln!("Error receiving event: {:?}", e);
                        handler_conn_err(e);

                        println!("Reconnecting in {:?}", sleep_duration);
                        std::thread::sleep(sleep_duration);
                    }
                }
            }

            Err(e) => {
                eprintln!(
                    "Connection error:{:?}\n Reconnecting in {:?}",
                    e, sleep_duration
                );
                std::thread::sleep(sleep_duration);
            }
        }
    }
}
