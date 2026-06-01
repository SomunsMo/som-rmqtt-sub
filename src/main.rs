/// 退出程序宏
#[macro_export]
macro_rules! err_exit {
    ($($arg:tt)*) => {{
        log::error!($($arg)*);
        log::info!("Exiting...\n");
        std::process::exit(1);
    }};
}

// 宏定义区域结束
// 宏定义要早于config模块引用，否则config模块内的代码无法使用上面的宏
// --------------------------

mod config;

use rumqttc::{Client, ConnectionError, Event, Incoming, MqttOptions, QoS, Transport};
use std::process::Command;
use std::time::Duration;

/// 处理连接错误
fn handler_conn_err(e: ConnectionError) {
    match e {
        ConnectionError::ConnectionRefused(_) => {
            // 认证相关错误，直接退出
            err_exit!("Connection refused: {:?}", e);
        }
        _ => {}
    }
}

/// 初始化MQTT客户端
fn init() -> (&'static config::mqtt_config::Config, MqttOptions) {
    // 先初始化日志系统
    config::log_config::init_logger();

    // 打印程序启动信息，防止log与上次运行的日志视觉粘连
    log::info!("======================================================");
    log::info!("MQTT Client Starting...");

    let config = &config::mqtt_config::MQTT_CONFIG;

    // MQTT 连接
    let mut mqtt_opts =
        MqttOptions::new(&config.mqtt.client_id, &config.mqtt.host, config.mqtt.port);
    mqtt_opts.set_credentials(&config.mqtt.username, &config.mqtt.password);
    mqtt_opts.set_keep_alive(Duration::from_secs(30));

    // TLS 配置
    if config.mqtt.use_tls {
        log::info!("SSL/TLS Enabled (MQTTS)");
        // 使用系统默认配置（信任系统根证书）
        mqtt_opts.set_transport(Transport::tls_with_default_config());
    } else {
        log::info!("SSL/TLS Disabled (MQTT)");
    }

    log::info!("======================================================");

    (config, mqtt_opts)
}

fn main() {
    let (config, mqtt_opts) = init();
    let (client, mut connection) = Client::new(mqtt_opts, 10);

    log::info!("Connecting to {}:{}", config.mqtt.host, config.mqtt.port);

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
                                log::info!("Connected successfully");

                                // 连接成功后，订阅所有主题
                                for subscription in &config.subscriptions {
                                    client
                                        .subscribe(&subscription.topic, QoS::AtMostOnce)
                                        .unwrap_or_else(|e| {
                                            err_exit!(
                                                "Failed to subscribe to topic '{}': {:?}",
                                                subscription.topic,
                                                e
                                            );
                                        });
                                    log::info!("- Subscribed to topic: {}", subscription.topic);
                                }

                                log::info!("");
                            }
                            Incoming::Publish(p) => {
                                let payload = String::from_utf8_lossy(&p.payload);
                                let msg = payload.trim();
                                let topic = &p.topic;
                                log::info!("[Topic: {}] {}", topic, msg);

                                // 查找对应的订阅配置并执行命令
                                for subscription in &config.subscriptions {
                                    if &subscription.topic == topic {
                                        if let Some(cmd) = subscription.commands.get(msg) {
                                            log::info!("[Execute] {}", cmd);
                                            let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
                                        }
                                        break;
                                    }
                                }
                            }
                            Incoming::Disconnect => {
                                log::error!("Disconnected from MQTT broker");
                            }
                            _ => {
                                // log::info!("Received unexpected packet: {:?}", incoming);
                            }
                        }
                    }
                    Ok(Event::Outgoing(_outgoing)) => {
                        // 处理 outgoing 事件（可选）
                        // log::info!("Outgoing event: {:?}", _outgoing);
                    }
                    Err(e) => {
                        log::error!("Event reception error: {:?}", e);
                        handler_conn_err(e);

                        log::info!("Reconnecting in {:?}", sleep_duration);
                        std::thread::sleep(sleep_duration);
                    }
                }
            }

            Err(e) => {
                log::error!("Connection error:{:?} ", e);
                log::info!("Reconnecting in {:?}", sleep_duration);
                std::thread::sleep(sleep_duration);
            }
        }
    }
}
