use crate::services::osd::{OsdEvent, OsdService, OsdStateChanged};
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;

pub struct OsdWidget {
    current_event: Option<OsdEvent>,
}

impl OsdWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let osd = OsdService::global(cx);

        cx.subscribe(&osd, move |this, _osd, event: &OsdStateChanged, cx| {
            this.current_event = event.event.clone();
            cx.notify();
        })
        .detach();

        Self { current_event: None }
    }
}

impl Render for OsdWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Si pas d'événement, retourner un div vide pour cacher la fenêtre layer shell
        if self.current_event.is_none() {
            return div().size_0().into_any_element();
        }

        let event = self.current_event.as_ref().unwrap();

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
                                    .w(relative(*level as f32 / 100.0))
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
                let icon_name = if *muted { "source-muted" } else { "source-high" };

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
                            .child(if *muted { "Microphone Muted" } else { "Microphone Active" }),
                    )
            }
            OsdEvent::CapsLock(enabled) => {
                let icon_name = if *enabled { "capslock-on" } else { "capslock-off" };
                let text = if *enabled { "Caps Lock On" } else { "Caps Lock Off" };

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

        div()
            .w(px(400.))
            .h(px(64.))
            .bg(theme.bg)
            .rounded(px(12.))
            .px_4()
            .py_3()
            .child(content)
            .into_any_element()
    }
}