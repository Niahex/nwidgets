use crate::services::osd::{OsdEvent, OsdService, OsdStateChanged};
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;
use std::time::Duration;

pub struct OsdWidget {
    osd: Entity<OsdService>,
}

impl OsdWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let osd = OsdService::global(cx);

        // Subscribe to OSD state changes
        cx.subscribe(&osd, |this, _osd, event: &OsdStateChanged, cx| {
            if event.visible {
                // Auto-hide après 2.5 secondes
                this.schedule_hide(cx);
            }
            cx.notify();
        })
        .detach();

        Self { osd }
    }

    fn schedule_hide(&self, cx: &mut Context<Self>) {
        let osd = self.osd.clone();
        cx.spawn(async move |_this, mut cx| {
            cx.background_executor()
                .timer(Duration::from_millis(2500))
                .await;

            let _ = osd.update(cx, |service, cx| {
                service.hide(cx);
            });
        })
        .detach();
    }
}

impl Render for OsdWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let osd = self.osd.read(cx);
        let visible = osd.is_visible();
        let event = osd.current_event();

        if !visible || event.is_none() {
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
                    .child(
                        Icon::new(icon_name)
                            .size(px(20.))
                            .color(text_color)
                    )
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
                                    .rounded(px(3.))
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
                                    .rounded(px(3.))
                            )
                    )
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_color)
                            .child(format!("{}", level))
                    )
            }
            OsdEvent::Microphone(muted) => {
                let icon_name = if muted {
                    "source-muted"
                } else {
                    "source-high"
                };

                div()
                    .flex()
                    .gap_3()
                    .items_center()
                    .child(
                        Icon::new(icon_name)
                            .size(px(20.))
                            .color(text_color)
                    )
                    .child(
                        // Barre de progression
                        div()
                            .w(px(240.))
                            .h(px(6.))
                            .relative()
                            .child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .w_full()
                                    .h_full()
                                    .bg(progress_bg)
                                    .rounded(px(3.))
                            )
                            .child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .w(relative(0.89))
                                    .h_full()
                                    .bg(progress_fg)
                                    .rounded(px(3.))
                            )
                    )
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_color)
                            .child("89")
                    )
            }
        };

        div()
            .w(px(400.))
            .h(px(64.))
            .bg(bg_color)
            .rounded(px(12.))
            .px_4()
            .py_3()
            .child(content)
            .into_any_element()
    }
}
