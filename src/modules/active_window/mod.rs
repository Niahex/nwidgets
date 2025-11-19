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
        // Si aucune fenêtre active, afficher l'icône de flocon de neige avec NixOS/Nia
        let (icon, class, title) = if let Some(active_window) = &self.active_window {
            // Tronquer le titre pour qu'il ne soit pas trop long (max 30 caractères)
            let truncated_title = if active_window.title.chars().count() > 30 {
                let truncated: String = active_window.title.chars().take(27).collect();
                format!("{}...", truncated)
            } else {
                active_window.title.clone()
            };
            ("🪟", active_window.class.clone(), truncated_title)
        } else {
            ("❄️", "NixOS".to_string(), "Nia".to_string())
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
            .child(div().text_color(rgb(FROST1)).text_base().child(icon))
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
