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

        let theme = cx.global::<crate::theme::Theme>();

        let content = match event {
            OsdEvent::Volume(icon_name, level, _muted) => {
                div()
                    .flex()
                    .gap_3()
                    .items_center()
                    .child(Icon::new(icon_name).size(px(20.)).color(theme.text))
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
                                    .bg(theme.hover)
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
                                    .bg(theme.accent_alt)
                                    .rounded(px(3.)),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(format!("{level}")),
                    )
            }
            OsdEvent::Microphone(muted) => {
                let icon_name = if muted { "source-muted" } else { "source-high" };

                div()
                    .flex()
                    .gap_3()
                    .items_center()
                    .child(Icon::new(icon_name).size(px(20.)).color(theme.text))
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
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
                    .child(Icon::new(icon_name).size(px(20.)).color(theme.text))
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(text),
                    )
            }
            OsdEvent::Clipboard => {
                div()
                    .flex()
                    .gap_3()
                    .items_center()
                    .justify_center()
                    .child(Icon::new("copy").size(px(20.)).color(theme.text))
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child("Copied to clipboard"),
                    )
            }
        };

        let base_div = div()
            .w(px(400.))
            .h(px(64.))
            .bg(theme.bg)
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