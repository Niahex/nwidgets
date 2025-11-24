use crate::icons;
use crate::services::hyprland::ActiveWindow;
use gtk::prelude::*;
use gtk4 as gtk;

#[derive(Clone)]
pub struct ActiveWindowModule {
    pub container: gtk::CenterBox,
    icon_container: gtk::CenterBox,
    text_container: gtk::CenterBox,
    icon: gtk::Image,
    class_label: gtk::Label,
    title_label: gtk::Label,
}

impl ActiveWindowModule {
    pub fn new() -> Self {
        // Container principal
        let container = gtk::CenterBox::new();
        container.add_css_class("active-window-widget");
        container.set_width_request(250);

        // CenterBox pour l'icône
        let icon_container = gtk::CenterBox::new();
        icon_container.set_width_request(64);

        let icon = icons::create_icon("nixos", 32);
        icon.add_css_class("active-window-icon");
        icon_container.set_center_widget(Some(&icon));

        // CenterBox pour le texte (class + title)
        let text_container = gtk::CenterBox::new();
        text_container.set_hexpand(true);

        let text_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        text_box.set_halign(gtk::Align::Center);
        text_box.set_valign(gtk::Align::Center);

        let class_label = gtk::Label::new(None);
        class_label.add_css_class("active-window-class");
        class_label.set_halign(gtk::Align::Center);

        let title_label = gtk::Label::new(None);
        title_label.add_css_class("active-window-title");
        title_label.set_halign(gtk::Align::Center);

        text_box.append(&class_label);
        text_box.append(&title_label);
        text_container.set_center_widget(Some(&text_box));

        container.set_start_widget(Some(&icon_container));
        container.set_center_widget(Some(&text_container));

        Self {
            container,
            icon_container,
            text_container,
            icon,
            class_label,
            title_label,
        }
    }

    pub fn update(&self, active_window: Option<ActiveWindow>) {
        let (icon_name, class, title) = if let Some(active_window) = &active_window {
            let title_before_dash = active_window
                .title
                .split(" - ")
                .next()
                .unwrap_or(&active_window.title)
                .trim()
                .to_string();

            let truncated_title = if title_before_dash.chars().count() > 30 {
                let truncated: String = title_before_dash.chars().take(27).collect();
                format!("{}...", truncated)
            } else {
                title_before_dash
            };

            let icon_name = match active_window.class.to_lowercase().as_str() {
                "firefox" => "firefox",
                "google-chrome" | "chromium" => "google-chrome",
                "code" | "vscode" => "code",
                "discord" => "discord",
                "steam" => "steam",
                "kitty" | "alacritty" | "terminal" => "terminal",
                "nautilus" | "thunar" => "folder",
                _ => "application-default-icon",
            };

            let display_class = active_window
                .class
                .split('-')
                .next()
                .unwrap_or(&active_window.class)
                .to_string();

            (icon_name, display_class, truncated_title)
        } else {
            ("nixos", "NixOS".to_string(), "Nia".to_string())
        };

        self.icon.set_icon_name(Some(icon_name));
        self.class_label.set_text(&class);
        self.title_label.set_text(&title);
    }
}
