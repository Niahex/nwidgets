use crate::services::mpris::{MprisService, MprisStateChanged, PlaybackStatus};
use gpui::prelude::*;
use gpui::*;
use std::cell::Cell;
use std::time::{Duration, Instant};

pub struct MprisModule {
    mpris: Entity<MprisService>,
    last_track_change: Cell<Instant>,
}

impl MprisModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mpris = MprisService::global(cx);

        cx.subscribe(&mpris, |_this, _mpris, _event: &MprisStateChanged, cx| {
            cx.notify();
        })
        .detach();

        Self {
            mpris,
            last_track_change: Cell::new(Instant::now() - Duration::from_secs(1)),
        }
    }
}

impl Render for MprisModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let player = self.mpris.read(cx).current_player();

        if let Some(player) = player {
            let mpris = self.mpris.clone();
            let title = player
                .metadata
                .title
                .unwrap_or_else(|| "No title".to_string());
            let artist = player.metadata.artist;
            let is_paused = player.status == PlaybackStatus::Paused;

            let last_track_change = self.last_track_change.clone();
            let mpris_for_click = mpris.clone();
            let mpris_for_scroll = mpris.clone();

            div()
                .id("mpris-module")
                .flex()
                .flex_col()
                .w(px(250.))
                .px_2()
                .py_1()
                .rounded_sm()
                .cursor_pointer()
                .when(is_paused, |this| {
                    this.text_color(rgba(0xd8dee980)) // Dimmed when paused
                })
                .when(!is_paused, |this| this.text_color(rgb(0xeceff4)))
                .hover(|style| style.bg(rgba(0x4c566a40)))
                // Click to play/pause
                .on_click(move |_event, _window, cx| {
                    mpris_for_click.read(cx).play_pause();
                })
                // Scroll handlers
                .on_scroll_wheel(move |event, window, cx| {
                    let mpris = mpris_for_scroll.read(cx);
                    let delta_pixels = event.delta.pixel_delta(window.line_height());

                    // Horizontal scroll for track navigation (with debounce)
                    if !delta_pixels.x.is_zero() {
                        let now = Instant::now();
                        let cooldown = Duration::from_millis(300);

                        if now.duration_since(last_track_change.get()) >= cooldown {
                            if delta_pixels.x < px(0.0) {
                                mpris.previous();
                            } else {
                                mpris.next();
                            }
                            last_track_change.set(now);
                        }
                    }

                    // Vertical scroll for volume (inverted: scroll up = volume up)
                    if !delta_pixels.y.is_zero() {
                        if delta_pixels.y < px(0.0) {
                            mpris.volume_down();
                        } else {
                            mpris.volume_up();
                        }
                    }
                })
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(title),
                )
                .when_some(artist, |this, artist_name| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(rgba(0xd8dee980)) // $snow0 with opacity
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(artist_name),
                    )
                })
                .into_any_element()
        } else {
            div().into_any_element()
        }
    }
}
