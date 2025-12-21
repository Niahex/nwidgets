use crate::services::osd::{OsdEvent, OsdService, OsdStateChanged};
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;

pub struct OsdWidget {
    osd: Entity<OsdService>,
}

impl OsdWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let osd = OsdService::global(cx);

        // On s'abonne juste pour rafraichir la vue, plus de timer ici
        cx.subscribe(&osd, move |_this, _osd, _event: &OsdStateChanged, cx| {
            cx.notify();
        })
        .detach();

        Self { osd }
    }
}

impl Render for OsdWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let osd = self.osd.read(cx);
        let visible = osd.is_visible();
        let event = osd.current_event();

        // Si on n'a jamais eu d'événement, on rend vide
        if event.is_none() {
            return div().into_any_element();
        }

        let event = event.unwrap();

        // Nord colors
        let bg_color = rgb(0x2e3440); // polar0
        let text_color = rgb(0xeceff4); // snow3
        let progress_bg = rgb(0x4c566a); // polar3
        let progress_fg = rgb(0x8fbcbb); // frost3

        let content = match event {
            OsdEvent::Volume(icon_name, level, _muted) => {
                div()
                    .flex()
                    .gap_3()
                    .items_center()
                    .child(Icon::new(icon_name).size(px(20.)).color(text_color))
                    .child(
                        // Barre de progression
                        div()
                            .w(px(240.))
                            .h(px(6.))
                            .relative()
                            .child(
                                // Background
                                div()
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .w_full()
                                    .h_full()
                                    .bg(progress_bg)
                                    .rounded(px(3.)),
                            )
                            .child(
                                // Foreground (filled)
                                div()
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .w(relative(level as f32 / 100.0))
                                    .h_full()
                                    .bg(progress_fg)
                                    .rounded(px(3.)),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_color)
                            .child(format!("{}", level)),
                    )
            }
            OsdEvent::Microphone(muted) => {
                let icon_name = if muted { "source-muted" } else { "source-high" };

                div()
                    .flex()
                    .gap_3()
                    .items_center()
                    .child(Icon::new(icon_name).size(px(20.)).color(text_color))
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_color)
                            .child(if muted { "Microphone Muted" } else { "Microphone Active" }),
                    )
            }
            OsdEvent::CapsLock(enabled) => {
                let icon_name = if enabled { "capslock-on" } else { "capslock-off" };
                let text = if enabled { "Caps Lock On" } else { "Caps Lock Off" };

                div()
                    .flex()
                    .gap_3()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(icon_name).size(px(20.)).color(text_color))
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_color)
                            .child(text),
                    )
            }
            OsdEvent::Clipboard => {
                div()
                    .flex()
                    .gap_3()
                    .items_center()
                    .justify_center()
                    .child(Icon::new("copy").size(px(20.)).color(text_color))
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_color)
                            .child("Copied to clipboard"),
                    )
            }
        };

        let base_div = div()
            .w(px(400.))
            .h(px(64.))
            .bg(bg_color)
            .rounded(px(12.))
            .px_4()
            .py_3()
            .child(content);

        if visible {
            base_div.visible().into_any_element()
        } else {
            base_div.invisible().into_any_element()
        }
    }
}