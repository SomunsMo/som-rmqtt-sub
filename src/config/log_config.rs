use colored::Colorize;
use env_logger::{Builder, Env};
use log::Level;
use std::io::Write;

// 初始化日志系统
pub fn init_logger() {
    let env = Env::default().default_filter_or("info");
    Builder::from_env(env)
        .format(|buf, record| {
            // 日志的时间信息（年/月/日 时:分:秒.毫秒）
            let timestamp = chrono::Local::now().format("%Y/%m/%d %H:%M:%S%.3f");

            // 使用 colored 添加颜色
            let colored_level = match record.level() {
                Level::Error => record.level().to_string().red().bold(),
                Level::Warn => record.level().to_string().yellow().bold(),
                Level::Info => record.level().to_string().green().bold(),
                Level::Debug => record.level().to_string().blue(),
                Level::Trace => record.level().to_string().cyan(),
            };

            writeln!(
                buf,
                "{} {:<5} | {}",
                timestamp,
                colored_level,
                record.args()
            )
        })
        .init();
}
