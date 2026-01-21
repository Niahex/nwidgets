use crate::services::ui::osd::{OsdEvent, OsdService, OsdStateChanged};
use crate::theme::ActiveTheme;
use crate::assets::Icon;
use crate::ui::components::{Slider, SliderState};
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

            // Mettre à jour target_volume depuis l'événement
            if let Some(OsdEvent::Volume(_, vol, _)) = &event.event {
                this.target_volume = *vol as f32;
            }

            cx.notify();
        })
        .detach();

        // Animation loop
        cx.spawn(async move |this, cx| loop {
            cx.background_executor()
                .timer(Duration::from_millis(8))
                .await;

            let _ = this.update(cx, |widget, cx| {
                if (widget.displayed_volume - widget.target_volume).abs() > 0.1 {
                    widget.displayed_volume +=
                        (widget.target_volume - widget.displayed_volume) * 0.7;
                    
                    // Update slider value
                    widget.volume_slider.update(cx, |slider, cx| {
                        slider.update_value(widget.displayed_volume, cx);
                    });
                    
                    cx.notify();
                }
            });
        })
        .detach();

        Self {
            current_event: initial_event,
            visible: initial_visible,
            displayed_volume: initial_volume,
            target_volume: initial_volume,
            volume_slider,
        }
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
        // Si pas visible ou pas d'événement, retourner un élément vide
        let Some(event) = self.current_event.as_ref() else {
            return div().into_any_element();
        };

        if !self.visible {
            return div().into_any_element();
        }

        let theme = cx.theme();

        let content = match event {
            OsdEvent::Volume(icon_name, _level, _muted) => {
                // Arrondir à 5 uniquement pour l'affichage du chiffre
                let display_val = ((self.displayed_volume / 5.0).round() * 5.0) as u8;

                div()
                    .flex()
                    .gap_3()
                    .items_center()
                    .child(Icon::new(icon_name).size(px(20.)).color(theme.text))
                    .child(
                        div()
                            .w(px(240.))
                            .child(Slider::new(&self.volume_slider).readonly(true))
                    )
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(format!("{display_val}")),
                    )
            }
            OsdEvent::Microphone(muted) => {
                let icon_name = if *muted {
                    "source-muted"
                } else {
                    "source-high"
                };

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
                            .child(if *muted {
                                "Microphone Muted"
                            } else {
                                "Microphone Active"
                            }),
                    )
            }
            OsdEvent::CapsLock(enabled) => {
                let icon_name = if *enabled {
                    "capslock-on"
                } else {
                    "capslock-off"
                };
                let text = if *enabled {
                    "Caps Lock On"
                } else {
                    "Caps Lock Off"
                };

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
            OsdEvent::Clipboard => div()
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
                ),
        };

        // Animation: fade in/out
        let is_visible = self.visible;
        let animation_id = if is_visible {
            "osd-fade-in"
        } else {
            "osd-fade-out"
        };

        // Si pas visible, retourner un élément vide
        if !is_visible {
            return div().into_any_element();
        }

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
                animation_id,
                Animation::new(Duration::from_millis(200)),
                move |this, delta| this.opacity(delta),
            )
            .into_any_element()
    }
}
