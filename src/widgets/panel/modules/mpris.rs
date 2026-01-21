use crate::theme::ActiveTheme;
use crate::services::mpris::{MprisService, MprisStateChanged, PlaybackStatus};
use gpui::prelude::*;
use gpui::*;
use std::time::{Duration, Instant};

pub struct MprisModule {
    mpris: Entity<MprisService>,
    scroll_acc_x: Pixels,
    scroll_acc_y: Pixels,
    last_track_change: Instant,
    // Cache
    cached_title: SharedString,
    cached_artist: Option<SharedString>,
    cached_status: PlaybackStatus,
}

impl MprisModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mpris = MprisService::global(cx);

        let (title, artist, status) = if let Some(player) = mpris.read(cx).current_player() {
            (
                player.metadata.title.unwrap_or_else(|| "No title".to_string()).into(),
                player.metadata.artist.map(|a| a.into()),
                player.status,
            )
        } else {
            ("No title".into(), None, PlaybackStatus::Stopped)
        };

        cx.subscribe(&mpris, |this, mpris, _event: &MprisStateChanged, cx| {
            this.update_cache(mpris.read(cx).current_player().as_ref());
            cx.notify();
        })
        .detach();

        Self {
            mpris,
            scroll_acc_x: px(0.0),
            scroll_acc_y: px(0.0),
            last_track_change: Instant::now(),
            cached_title: title,
            cached_artist: artist,
            cached_status: status,
        }
    }

    fn update_cache(&mut self, player: Option<&crate::services::mpris::MprisPlayer>) {
        if let Some(player) = player {
            self.cached_title = player.metadata.title.clone().unwrap_or_else(|| "No title".to_string()).into();
            self.cached_artist = player.metadata.artist.clone().map(|a| a.into());
            self.cached_status = player.status.clone();
        } else {
            self.cached_title = "No title".into();
            self.cached_artist = None;
            self.cached_status = PlaybackStatus::Stopped;
        }
    }
}

impl Render for MprisModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let player = self.mpris.read(cx).current_player();

        if player.is_some() {
            let is_paused = self.cached_status == PlaybackStatus::Paused;

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
                .when(!is_paused, |this| {
                    this.text_color(cx.theme().text)
                })
                .hover(|style| style.bg(rgba(0x4c566a40)))
                // Click to play/pause
                .on_click(cx.listener(|this, _event, _window, cx| {
                    this.mpris.update(cx, |mpris, cx| mpris.play_pause(cx));
                }))
                // Scroll handlers
                .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, window, cx| {
                    let delta_pixels = event.delta.pixel_delta(window.line_height());

                    // Accumulate deltas
                    this.scroll_acc_x += delta_pixels.x;
                    this.scroll_acc_y += delta_pixels.y;

                    let x_threshold = px(50.0);
                    let y_threshold = px(15.0);

                    // Horizontal scroll for track navigation
                    if this.scroll_acc_x >= x_threshold || this.scroll_acc_x <= -x_threshold {
                        let now = Instant::now();
                        if now.duration_since(this.last_track_change) >= Duration::from_millis(500)
                        {
                            if this.scroll_acc_x < px(0.0) {
                                this.mpris.update(cx, |mpris, cx| mpris.previous(cx));
                            } else {
                                this.mpris.update(cx, |mpris, cx| mpris.next(cx));
                            }
                            this.last_track_change = now;
                        }
                        // Reset accumulator even if cooldown prevented skip to avoid cumulative lag
                        this.scroll_acc_x = px(0.0);
                    }

                    // Vertical scroll for volume
                    while this.scroll_acc_y >= y_threshold || this.scroll_acc_y <= -y_threshold {
                        if this.scroll_acc_y < px(0.0) {
                            this.mpris.update(cx, |mpris, _cx| mpris.volume_down());
                            this.scroll_acc_y += y_threshold;
                        } else {
                            this.mpris.update(cx, |mpris, _cx| mpris.volume_up());
                            this.scroll_acc_y -= y_threshold;
                        }
                    }
                }))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(self.cached_title.clone()),
                )
                .when_some(self.cached_artist.clone(), |this, artist_name| {
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
