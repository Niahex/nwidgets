use crate::services::osd::{OsdEvent, OsdService, OsdStateChanged};
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;
use std::time::Duration;

pub struct OsdWidget {
    current_event: Option<OsdEvent>,
    visible: bool,
}

impl OsdWidget {
    pub fn new(cx: &mut Context<Self>, initial_event: Option<OsdEvent>, initial_visible: bool) -> Self {
        let osd = OsdService::global(cx);
        
        cx.subscribe(&osd, move |this, _osd, event: &OsdStateChanged, cx| {
            // Avoid reading the service here to prevent RefCell panic
            this.current_event = event.event.clone();
            this.visible = event.visible;
            cx.notify();
        })
        .detach();

        Self { 
            current_event: initial_event, 
            visible: initial_visible 
        }
    }
}

impl Render for OsdWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Always render the structure, just manage opacity
        // If we have no event yet at all, render transparently
        if self.current_event.is_none() {
             return div().id("osd-root").size_0().into_any_element();
        }

        let event = self.current_event.as_ref().unwrap();
        let theme = cx.global::<crate::theme::Theme>();

        // Define transition style for the whole OSD
        let _opacity = if self.visible { 1.0 } else { 0.0 };

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

        // If not visible, we want to animate out, so we start at 1.0 (delta 0) and go to 0.0 (delta 1).
        // If visible, we want to animate in, so we start at 0.0 (delta 0) and go to 1.0 (delta 1).
        let is_visible = self.visible;
        let animation_id = if is_visible { "osd-fade-in" } else { "osd-fade-out" };
        
        div()
            .id("osd-root")
            .w(px(400.))
            .h(px(64.))
            .bg(theme.bg)
            .rounded(px(12.))
            .px_4()
            .py_3()
            // Initial opacity matches target state to avoid flash if animation doesn't run or finishes
            .opacity(if is_visible { 1.0 } else { 0.0 })
            .child(content)
            .with_animation(
                animation_id, 
                Animation::new(Duration::from_millis(200)), 
                move |this, delta| {
                    let opacity = if is_visible { delta } else { 1.0 - delta };
                    this.opacity(opacity)
                }
            )
            .into_any_element()
    }
}