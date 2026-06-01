use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::sync::LazyLock;

// --------------------------
// JSON5 配置结构
// --------------------------
#[derive(Debug, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub subscriptions: Vec<SubscriptionConfig>,
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
}

#[derive(Debug, Deserialize)]
pub struct SubscriptionConfig {
    pub topic: String,
    pub commands: HashMap<String, String>,
}

// --------------------------

// MQTT 配置（全局单例 | 懒加载）
pub static MQTT_CONFIG: LazyLock<Config> = LazyLock::new(|| {
    // 读取配置
    let config_path = parse_config_path();
    log::info!("Config path: {:?}", config_path);

    let config_str = read_to_string(&config_path).unwrap_or_else(|_| {
        crate::err_exit!("Could not find config file at {:?}", config_path);
    });
    let config: Config = json5::from_str(&config_str).unwrap_or_else(|_| {
        crate::err_exit!("Invalid configuration format in {:?}", config_path);
    });
    config
});

// --------------------------

/// 解析配置文件路径
pub fn parse_config_path() -> PathBuf {
    let args: Vec<String> = env::args().collect();

    // 指定配置文件的多种参数方式
    let config_args = ["-config", "-c", "--config", "/config"];
    for i in 0..args.len() {
        // 第二条件防止启动参数最后一个是“指定配置文件”的参数标识（如 xxx -c）
        // 这个时候访问i+1会导致越界访问从而崩溃
        if config_args.contains(&&*args[i]) && i + 1 < args.len() {
            return PathBuf::from(&args[i + 1]);
        }
    }

    // 默认从当前目录读取
    let path = PathBuf::from("config.json5");
    path
}
