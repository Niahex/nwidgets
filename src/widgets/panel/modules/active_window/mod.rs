use crate::services::hyprland::ActiveWindow;
use crate::theme::icons;
use gtk::prelude::*;
use gtk4 as gtk;

#[derive(Clone)]
pub struct ActiveWindowModule {
    pub container: gtk::Box,
    icon_label: gtk::Label,
    class_label: gtk::Label,
    title_label: gtk::Label,
}

impl ActiveWindowModule {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        container.add_css_class("active-window-widget");
        container.set_width_request(256); // 64 * 4

        let icon_label = gtk::Label::new(None);
        icon_label.add_css_class("active-window-icon"); // Assuming you have a CSS file with this class

        let class_label = gtk::Label::new(None);
        class_label.add_css_class("active-window-class"); // Assuming you have a CSS file with this class

        let title_label = gtk::Label::new(None);
        title_label.add_css_class("active-window-title"); // Assuming you have a CSS file with this class

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content_box.append(&class_label);
        content_box.append(&title_label);

        container.append(&icon_label);
        container.append(&content_box);

        Self {
            container,
            icon_label,
            class_label,
            title_label,
        }
    }

    pub fn update(&self, active_window: Option<ActiveWindow>) {
        let (icon, class, title) = if let Some(active_window) = &active_window {
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

            let icon = icons::ICONS.get_for_class(&active_window.class);

            let display_class = active_window
                .class
                .split('-')
                .next()
                .unwrap_or(&active_window.class)
                .to_string();

            (icon, display_class, truncated_title)
        } else {
            (icons::ICONS.nixos, "NixOS".to_string(), "Nia".to_string())
        };

        self.icon_label.set_text(icon);
        self.class_label.set_text(&class);
        self.title_label.set_text(&title);
    }
}
