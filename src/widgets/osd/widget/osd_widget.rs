use crate::components::SliderState;
use crate::theme::ActiveTheme;
use crate::widgets::osd::service::OsdService;
use crate::widgets::osd::types::{OsdEvent, OsdStateChanged, ANIMATION_FRAME_MS, ANIMATION_SMOOTHNESS};
use crate::widgets::osd::widget::{capslock_renderer, clipboard_renderer, volume_renderer};
use gpui::prelude::*;
use gpui::*;
use std::time::Duration;

pub struct OsdWidget {
    current_event: Option<OsdEvent>,
    visible: bool,
    displayed_volume: f32,
    target_volume: f32,
    volume_slider: Entity<SliderState>,
}

impl OsdWidget {
    pub fn new(
        cx: &mut Context<Self>,
        initial_event: Option<OsdEvent>,
        initial_visible: bool,
    ) -> Self {
        let osd = OsdService::global(cx);
        let initial_volume = Self::get_initial_volume();

        let volume_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .default_value(initial_volume)
        });

        cx.subscribe(&osd, move |this, _osd, event: &OsdStateChanged, cx| {
            this.current_event = event.event.clone();
            this.visible = event.visible;

            if let Some(OsdEvent::Volume(_, vol, _)) = &event.event {
                this.target_volume = *vol as f32;
            }

            cx.notify();
        })
        .detach();

        Self::start_animation_loop(volume_slider.clone(), cx);

        Self {
            current_event: initial_event,
            visible: initial_visible,
            displayed_volume: initial_volume,
            target_volume: initial_volume,
            volume_slider,
        }
    }

    fn start_animation_loop(volume_slider: Entity<SliderState>, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| loop {
            cx.background_executor()
                .timer(Duration::from_millis(ANIMATION_FRAME_MS))
                .await;

            let _ = this.update(cx, |widget, cx| {
                if (widget.displayed_volume - widget.target_volume).abs() > 0.1 {
                    widget.displayed_volume +=
                        (widget.target_volume - widget.displayed_volume) * ANIMATION_SMOOTHNESS;

                    widget.volume_slider.update(cx, |slider, cx| {
                        slider.update_value(widget.displayed_volume, cx);
                    });

                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn get_initial_volume() -> f32 {
        if let Ok(output) = std::process::Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
        {
            if let Ok(text) = String::from_utf8(output.stdout) {
                if let Some(vol_str) = text.split_whitespace().nth(1) {
                    if let Ok(vol) = vol_str.parse::<f32>() {
                        return (vol * 100.0).clamp(0.0, 100.0);
                    }
                }
            }
        }
        50.0
    }
}

impl Render for OsdWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(event) = self.current_event.as_ref() else {
            return div().into_any_element();
        };

        if !self.visible {
            return div().into_any_element();
        }

        let theme = cx.theme().clone();

        let content = match event {
            OsdEvent::Volume(icon_name, _level, _muted) => {
                volume_renderer::render_volume(icon_name, self.displayed_volume, &self.volume_slider, &theme)
                    .into_any_element()
            }
            OsdEvent::Microphone(muted) => {
                let icon_name = if *muted { "source-muted" } else { "source-high" };
                let text = if *muted { "Microphone Muted" } else { "Microphone Active" };

                div()
                    .flex()
                    .gap_3()
                    .items_center()
                    .child(crate::assets::Icon::new(icon_name).size(px(20.)).color(theme.text))
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(text),
                    )
                    .into_any_element()
            }
            OsdEvent::CapsLock(enabled) => capslock_renderer::render_capslock(*enabled, &theme).into_any_element(),
            OsdEvent::Clipboard => clipboard_renderer::render_clipboard(&theme).into_any_element(),
        };

        div()
            .id("osd-root")
            .w(px(400.))
            .h(px(64.))
            .bg(theme.bg)
            .rounded(px(12.))
            .border_1()
            .border_color(theme.accent_alt.opacity(0.25))
            .shadow_lg()
            .flex()
            .items_center()
            .justify_center()
            .px_4()
            .py_3()
            .child(content)
            .with_animation(
                "osd-fade-in",
                Animation::new(Duration::from_millis(200)),
                |this, delta| this.opacity(delta),
            )
            .into_any_element()
    }
}
