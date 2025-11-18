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

    pub fn render(&self) -> Option<impl IntoElement> {
        self.active_window.as_ref().map(|active_window| {
            // Tronquer le titre pour qu'il ne soit pas trop long (max 30 caractères)
            let truncated_title = if active_window.title.len() > 30 {
                format!("{}...", &active_window.title[..27])
            } else {
                active_window.title.clone()
            };

            div()
                .w_64()  // Largeur fixe
                .h_10()  // Hauteur légèrement plus grande pour 2 lignes
                .px_3()
                .py_1()
                .bg(rgb(POLAR2))
                .rounded_md()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                // Icône
                .child(
                    div()
                        .text_color(rgb(FROST1))
                        .text_base()
                        .child("🪟")
                )
                // Contenu (classe + titre)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .justify_center()
                        .gap_0()
                        // Classe (plus petite)
                        .child(
                            div()
                                .text_color(rgb(POLAR3))
                                .text_xs()
                                .child(active_window.class.clone())
                        )
                        // Titre de l'application
                        .when(!truncated_title.is_empty(), |this| {
                            this.child(
                                div()
                                    .text_color(rgb(SNOW0))
                                    .text_sm()
                                    .child(truncated_title)
                            )
                        })
                )
        })
    }
}
