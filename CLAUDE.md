# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

- **Native build (debug):** `cargo build`
- **Native build (release):** `cargo build --release`
- **Cross-compile for mipsel (OpenWrt):** `cross build --target mipsel-unknown-linux-musl --release -Z build-std=core,std,alloc,panic_abort`
- **Check compilation errors quickly:** `cargo check`
- **Run locally:** `cargo run --release`
- **Compress binary with UPX (after cross-build):** `upx --ultra-brute --no-backup target/<triple>/release/som-rmqtt-sub`
- **Config location (default):** `./config.json5` or pass via `-c <path>`, `-config <path>`, `--config <path>`, `/config <path>`
- **No tests** exist in this project currently.

## Project Architecture

A minimal MQTT subscriber written in Rust that connects to an MQTT broker, subscribes to topics, and executes shell commands on matching messages. Designed to run on storage-constrained OpenWrt routers (mipsel).

### Core Flow

```
main.rs ──init()──▶ config::log_config::init_logger()
         │            └─ env_logger + colored (custom timestamp/level format)
         │
         └──init()──▶ config::mqtt_config::MQTT_CONFIG (LazyLock<Config>)
                        └─ Parse config.json5 (json5 format with comments)
                           └─ Struct: MqttConfig + Vec<SubscriptionConfig>
                              Each SubscriptionConfig: topic + HashMap<message, shell_command>
```

### Runtime Loop (main.rs)

1. Initialize logger and config (global LazyLock singleton)
2. Create `rumqttc::Client` with MQTT options (TLS optional via `use_tls`)
3. `connection.recv()` event loop:
   - `ConnAck` → subscribe to all configured topics
   - `Publish` → look up topic + message payload → spawn matching shell command via `Command::new("sh").arg("-c")`
   - `ConnectionError` (non-refused) → sleep 5s and reconnect
   - `ConnectionRefused` → log error and exit (auth failures are fatal)

### Key Dependencies

| Crate | Purpose |
|-------|---------|
| `rumqttc` | MQTT client (with native-tls + rustls) |
| `json5` | Config file with JSON5 (supports comments) |
| `serde` | Config deserialization |
| `colored` | Colorized log output |
| `chrono` | Timestamp formatting in logs |
| `env_logger` | Log framework |
| `openssl` (vendored) | Required for cross-compilation TLS |

### Release Profile

Optimized for size: `opt-level = "z"`, `lto = true`, `strip = true`, `codegen-units = 1`, `panic = "abort"`.

### Config Structure (config.json5)

```json5
{
  mqtt: {
    host: "bemfa.com",
    port: 9501,
    use_tls: false,
    username: "",
    password: "",
    client_id: "<private-key>",
  },
  subscriptions: [
    {
      topic: "DeviceTopic006",
      commands: {
        "on": "mkdir test_received_on",
        "off": "mkdir test_received_off",
      }
    }
  ]
}
```

### OpenWrt Deployment

`sh/create service.sh` handles end-to-end deployment:
- Copies binary to `/usr/bin/`
- Copies/downloads config to `/etc/som-rmqtt-sub/config.json5`
- Generates `wrapper.sh` with stdout/stderr redirect to log file
- Creates `/etc/init.d/som-rmqtt-sub` with OpenWrt procd service (auto-respawn)
- Sets up cron job for 7-day log rotation

### Cross-Compilation

Cross-compilation target configured in `Cross.toml` using `ghcr.io/cross-rs/mipsel-unknown-linux-musl` image. Build with `-Z build-std=core,std,alloc,panic_abort` when using custom panic=abort profile. Refer to README.md for full cross-build targets matrix (Windows/MacOS/Linux/ARM/mipsel).

### Memory Note

This project is heavily authored in Chinese (comments, documentation, README). Config file paths and error macro (`err_exit!`) are defined before the `config` module so they're available globally.
