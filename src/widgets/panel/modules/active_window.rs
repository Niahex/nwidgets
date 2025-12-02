use crate::services::chat::ChatState;
use crate::services::hyprland::ActiveWindow;
use crate::utils::icons;
use gtk::prelude::*;
use gtk4 as gtk;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct ActiveWindowModule {
    pub container: gtk::CenterBox,
    icon_container: gtk::CenterBox,
    text_container: gtk::CenterBox,
    icon: gtk::Image,
    class_label: gtk::Label,
    title_label: gtk::Label,
    // État pour savoir si le chat est visible
    chat_state: Rc<RefCell<ChatState>>,
    hyprland_window: Rc<RefCell<Option<ActiveWindow>>>,
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

        let icon = icons::create_icon_with_size("test", Some(48));
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
            chat_state: Rc::new(RefCell::new(ChatState::default())),
            hyprland_window: Rc::new(RefCell::new(None)),
        }
    }

    /// Met à jour l'état du chat
    pub fn update_chat_state(&self, chat_state: ChatState) {
        *self.chat_state.borrow_mut() = chat_state;
        self.refresh_display();
    }

    /// Met à jour la fenêtre Hyprland active
    pub fn update_hyprland_window(&self, active_window: Option<ActiveWindow>) {
        *self.hyprland_window.borrow_mut() = active_window;
        self.refresh_display();
    }

    /// Rafraîchit l'affichage en fonction de l'état actuel
    fn refresh_display(&self) {
        let chat_state = self.chat_state.borrow();
        let hyprland_window = self.hyprland_window.borrow();

        // Si le chat est visible, afficher l'info du chat
        if chat_state.is_visible && !chat_state.selected_site_name.is_empty() {
            let icon_name = self.get_chat_icon(&chat_state.selected_site_name);
            let title = &chat_state.selected_site_name;

            if let Some(paintable) = icons::get_paintable_with_size(icon_name, Some(48)) {
                self.icon.set_paintable(Some(&paintable));
            }
            self.class_label.set_text("Chat");
            self.title_label.set_text(title);
        } else {
            // Sinon, afficher l'info de la fenêtre Hyprland
            self.display_hyprland_window(hyprland_window.as_ref());
        }
    }

    fn get_chat_icon(&self, site_name: &str) -> &str {
        match site_name {
            "Gemini" => "gemini",
            "DeepSeek" => "deepseek",
            "AI Studio" => "ai-studio",
            "DuckDuckGo AI" => "duckduckgo",
            _ => "chat",
        }
    }

    fn display_hyprland_window(&self, active_window: Option<&ActiveWindow>) {
        let (icon_name, class, title) = if let Some(active_window) = active_window {
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
                "firefox" | "zen-twilight" => "firefox",
                "discord" | "vesktop" => "discord",
                "steam" => "steam",
                "vlc" => "vlc",
                "org.keepassxc.keepassxc" => "keepassxc",
                "spotify" => "spotify",
                "org.gnome.nautilus" => "file-manager",
                "org.inkscape.inkscape" => "inkscape",
                "kitty" | "alacritty" | "terminal" => "terminal",
                "dev.zed.zed" => "zeditor",
                _ => {
                    println!("DEBUG: Unknown window class: {}", active_window.class);
                    "test"
                }
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

        if let Some(paintable) = icons::get_paintable_with_size(icon_name, Some(48)) {
            self.icon.set_paintable(Some(&paintable));
        }
        self.class_label.set_text(&class);
        self.title_label.set_text(&title);
    }

    /// Méthode de compatibilité - redirige vers update_hyprland_window
    pub fn update(&self, active_window: Option<ActiveWindow>) {
        self.update_hyprland_window(active_window);
    }
}
