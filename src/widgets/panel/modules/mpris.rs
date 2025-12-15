use gpui::prelude::*;
use gpui::*;
use crate::services::mpris::{MprisService, MprisStateChanged, PlaybackStatus};

pub struct MprisModule {
    mpris: Entity<MprisService>,
}

impl MprisModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mpris = MprisService::global(cx);

        cx.subscribe(&mpris, |_this, _mpris, _event: &MprisStateChanged, cx| {
            cx.notify();
        })
        .detach();

        Self { mpris }
    }
}

impl Render for MprisModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let player = self.mpris.read(cx).current_player();
        let mpris = self.mpris.clone();

        if let Some(player) = player {
            let icon = match player.status {
                PlaybackStatus::Playing => "▶",
                PlaybackStatus::Paused => "⏸",
                PlaybackStatus::Stopped => "⏹",
            };

            let title = player.metadata.title.unwrap_or_else(|| "No title".to_string());
            let artist = player.metadata.artist;

            let mpris_prev = mpris.clone();
            let mpris_play = mpris.clone();
            let mpris_next = mpris.clone();

            div()
                .flex()
                .gap_2()
                .items_center()
                .max_w(px(250.))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(title)
                        )
                        .when_some(artist, |this, artist_name| {
                            this.child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x9399b2))
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(artist_name)
                            )
                        })
                )
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .child(
                            div()
                                .id("mpris-prev")
                                .px_1()
                                .rounded_sm()
                                .hover(|style| style.bg(rgb(0x313244)))
                                .cursor_pointer()
                                .on_click(move |_event, _window, cx| {
                                    mpris_prev.read(cx).previous();
                                })
                                .child("⏮")
                        )
                        .child(
                            div()
                                .id("mpris-play-pause")
                                .px_1()
                                .rounded_sm()
                                .hover(|style| style.bg(rgb(0x313244)))
                                .cursor_pointer()
                                .on_click(move |_event, _window, cx| {
                                    mpris_play.read(cx).play_pause();
                                })
                                .child(icon)
                        )
                        .child(
                            div()
                                .id("mpris-next")
                                .px_1()
                                .rounded_sm()
                                .hover(|style| style.bg(rgb(0x313244)))
                                .cursor_pointer()
                                .on_click(move |_event, _window, cx| {
                                    mpris_next.read(cx).next();
                                })
                                .child("⏭")
                        )
                )
        } else {
            div()
                .text_xs()
                .text_color(rgb(0x6c7086))
                .child("No media playing")
        }
    }
}
