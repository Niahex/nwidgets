use crate::services::hyprland::ActiveWindow;
use crate::theme::*;
use gpui::{div, prelude::*, rgb};

pub struct ActiveWindowModule {
    active_window: Option<ActiveWindow>,
}

impl ActiveWindowModule {
    pub fn new(active_window: Option<ActiveWindow>) -> Self {
        Self { active_window }
    }

    pub fn update(&mut self, active_window: Option<ActiveWindow>) {
        self.active_window = active_window;
    }

    pub fn render(&self) -> impl IntoElement {
        // Si aucune fenêtre active, afficher l'icône NixOS
        let (icon, class, title) = if let Some(active_window) = &self.active_window {
            // Extraire seulement ce qui est avant le premier "-" dans le titre
            let title_before_dash = active_window.title
                .split(" - ")
                .next()
                .unwrap_or(&active_window.title)
                .trim()
                .to_string();

            // Tronquer le titre pour qu'il ne soit pas trop long (max 30 caractères)
            let truncated_title = if title_before_dash.chars().count() > 30 {
                let truncated: String = title_before_dash.chars().take(27).collect();
                format!("{}...", truncated)
            } else {
                title_before_dash
            };

            // Mapper la classe de fenêtre à une icône appropriée
            let icon = match active_window.class.to_lowercase().as_str() {
                class if class.contains("vesktop") || class.contains("discord") => icons::VESKTOP,
                class if class.contains("zen") => icons::FIREFOX,
                class if class.contains("zed") => icons::ZED,
                class if class.contains("davinci") => icons::DAVINCI,
                class if class.contains("vlc") => icons::VLC,
                class
                    if class.contains("1password")
                        || class.contains("keepass")
                        || class.contains("bitwarden") =>
                {
                    icons::PASSWORD
                }
                class if class.contains("rofi") || class.contains("nlauncher") => icons::LAUNCHER,
                class if class.contains("steam") => icons::STEAM,
                class
                    if class.contains("game")
                        || class.contains("minecraft")
                        || class.contains("lutris") =>
                {
                    icons::GAME
                }
                class
                    if class.contains("kitty")
                        || class.contains("alacritty")
                        || class.contains("wezterm")
                        || class.contains("terminal") =>
                {
                    icons::TERMINAL
                }
                class if class.contains("inkscape") => icons::INKSCAPE,
                class if class.contains("obs") || class.contains("stream") => icons::STREAM,
                _ => icons::WINDOW, // Icône par défaut pour les fenêtres non reconnues
            };

            // Extraire seulement ce qui est avant le "-" dans le nom de la classe
            let display_class = active_window.class
                .split('-')
                .next()
                .unwrap_or(&active_window.class)
                .to_string();

            (icon, display_class, truncated_title)
        } else {
            (icons::NIXOS, "NixOS".to_string(), "Nia".to_string())
        };

        div()
            .w_64() // Largeur fixe
            .h_10() // Hauteur légèrement plus grande pour 2 lignes
            .px_3()
            .py_1()
            .rounded_md()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            // Icône
            .child(div().text_color(rgb(FROST0)).text_base().child(icon))
            // Contenu (classe + titre)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap_0()
                    // Classe (plus petite)
                    .child(div().text_color(rgb(SNOW0)).text_xs().child(class))
                    // Titre de l'application
                    .when(!title.is_empty(), |this| {
                        this.child(div().text_color(rgb(SNOW0)).text_sm().child(title))
                    }),
            )
    }
}
