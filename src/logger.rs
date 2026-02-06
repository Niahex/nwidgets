pub fn init() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .filter_module("blade_graphics", log::LevelFilter::Warn)
        .filter_module("naga", log::LevelFilter::Warn)
        .filter_module("zbus", log::LevelFilter::Warn)
        .filter_module("gpui::platform", log::LevelFilter::Warn)
        .filter_module("gpui::window", log::LevelFilter::Off)
        .filter_module("usvg::parser::svgtree", log::LevelFilter::Error)
        .format(|buf, record| {
            use std::io::Write;

            let level_str = match record.level() {
                log::Level::Error => "\x1b[1;31mERROR\x1b[0m",
                log::Level::Warn => "\x1b[1;33mWARN \x1b[0m",
                log::Level::Info => "\x1b[32mINFO \x1b[0m",
                log::Level::Debug => "\x1b[36mDEBUG\x1b[0m",
                log::Level::Trace => "\x1b[37mTRACE\x1b[0m",
            };

            let module = record.module_path().unwrap_or("unknown");
            let category = if module.starts_with("nwidgets::services") {
                format!("service::{}", module.strip_prefix("nwidgets::services::").unwrap_or(module))
            } else if module.starts_with("nwidgets::widgets") {
                format!("widget::{}", module.strip_prefix("nwidgets::widgets::").unwrap_or(module))
            } else if module.starts_with("nwidgets::components") {
                format!("component::{}", module.strip_prefix("nwidgets::components::").unwrap_or(module))
            } else if module.starts_with("nwidgets") {
                module.strip_prefix("nwidgets::").unwrap_or(module).to_string()
            } else {
                module.to_string()
            };

            writeln!(
                buf,
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                level_str,
                category,
                record.args()
            )
        })
        .init();
}
