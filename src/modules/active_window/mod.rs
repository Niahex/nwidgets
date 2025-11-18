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
            div()
                .h_8()
                .px_3()
                .bg(rgb(POLAR2))
                .rounded_md()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .text_color(rgb(SNOW0))
                .text_sm()
                .child(format!("🪟 {}", active_window.class))
                .when(!active_window.title.is_empty(), |this| {
                    this.child(
                        div()
                            .text_color(rgb(POLAR3))
                            .child(format!("- {}", active_window.title)),
                    )
                })
        })
    }
}
