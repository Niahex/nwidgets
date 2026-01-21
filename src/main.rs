mod app;
mod assets;
mod cli;
mod components;
mod logger;
mod nwidgets;
mod services;
mod theme;
mod widgets;
mod windows;

use gpui::*;


fn main() {
    logger::init();

    let args: Vec<String> = std::env::args().collect();

    // CEF subprocess
    if args.iter().any(|a| a.starts_with("--type=")) {
        cli::init_cef();
        return;
    }

    // CLI commands
    if args.len() > 1 {
        cli::handle_command(&args[1]);
        return;
    }

    // Main application
    Application::new()
        .with_assets(assets::Assets::new(assets::determine_assets_path()))
        .run(|cx: &mut App| {
            cli::init_cef();
            gpui_tokio::init(cx);
            cx.set_global(theme::Theme::nord_dark());

            nwidgets::run(cx);
        });
}
