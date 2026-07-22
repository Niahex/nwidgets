use std::time::Duration;
use gpui::*;
use gpui_component::corner::{Corner, CornerPosition};
use gpui_component::Icon;
use nwidgets_service_audio::{AudioService, AudioStateChanged};
use nwidgets_service_clipboard::{ClipboardChanged, ClipboardService};
use nwidgets_service_lock::{LockMonitor, LockStateChanged};

const CORNER_RADIUS: f32 = 12.0;
const DISPLAY_DURATION_MS: u64 = 2000;

#[derive(Debug, Clone, PartialEq)]
pub enum OsdEvent {
    Volume { volume: u8, muted: bool },
    Microphone { muted: bool },
    CapsLock { enabled: bool },
    Clipboard { content: String },
}

pub struct OsdView {
    window_handle: Option<AnyWindowHandle>,
    event: Option<OsdEvent>,
    visible: bool,
    _hide_task: Option<Task<()>>,
    last_sink_vol: u8,
    last_sink_muted: bool,
    last_source_muted: bool,
}

impl OsdView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let audio = AudioService::global(cx);
        let lock_monitor = LockMonitor::init(cx);
        let clipboard = ClipboardService::global(cx);

        let initial_audio = audio.read(cx).state.clone();

        let view = Self {
            window_handle: None,
            event: None,
            visible: false,
            _hide_task: None,
            last_sink_vol: initial_audio.sink_volume,
            last_sink_muted: initial_audio.sink_muted,
            last_source_muted: initial_audio.source_muted,
        };

        // Subscribe AudioService
        cx.subscribe(&audio, |this, service, _: &AudioStateChanged, cx| {
            let state = &service.read(cx).state;
            let vol_changed = state.sink_volume != this.last_sink_vol;
            let sink_mute_changed = state.sink_muted != this.last_sink_muted;
            let source_mute_changed = state.source_muted != this.last_source_muted;

            if vol_changed || sink_mute_changed {
                this.last_sink_vol = state.sink_volume;
                this.last_sink_muted = state.sink_muted;
                this.show_event(
                    OsdEvent::Volume {
                        volume: state.sink_volume,
                        muted: state.sink_muted,
                    },
                    cx,
                );
            } else if source_mute_changed {
                this.last_source_muted = state.source_muted;
                this.show_event(OsdEvent::Microphone { muted: state.source_muted }, cx);
            }
        })
        .detach();

        // Subscribe LockMonitor (CapsLock)
        cx.subscribe(&lock_monitor, |this, _, event: &LockStateChanged, cx| {
            this.show_event(OsdEvent::CapsLock { enabled: event.capslock_enabled }, cx);
        })
        .detach();

        // Subscribe ClipboardService
        cx.subscribe(&clipboard, |this, service, _: &ClipboardChanged, cx| {
            if let Some(entry) = service.read(cx).history.front() {
                this.show_event(
                    OsdEvent::Clipboard {
                        content: entry.content.clone(),
                    },
                    cx,
                );
            }
        })
        .detach();

        view
    }

    pub fn set_window_handle(&mut self, handle: AnyWindowHandle) {
        self.window_handle = Some(handle);
    }

    pub fn show_event(&mut self, event: OsdEvent, cx: &mut Context<Self>) {
        self.event = Some(event);
        self.visible = true;
        cx.notify();

        if let Some(ref handle) = self.window_handle {
            let handle = handle.clone();
            let _ = handle.update(cx, |_, window, _| {
                window.set_layer(gpui::layer_shell::Layer::Overlay);
                window.set_input_region(None);
                window.resize(size(px(340.0), px(54.0)));
            });
        }

        // Timer de masquage auto
        self._hide_task = Some(cx.spawn(|this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                cx.background_executor()
                    .timer(Duration::from_millis(DISPLAY_DURATION_MS))
                    .await;
                let _ = this.update(&mut cx, |this, cx| {
                    this.hide(cx);
                });
            }
        }));
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.visible = false;
        cx.notify();
        if let Some(ref handle) = self.window_handle {
            let handle = handle.clone();
            let _ = handle.update(cx, |_, window, _| {
                window.set_layer(gpui::layer_shell::Layer::Background);
                window.set_input_region(None);
                window.set_keyboard_interactivity(gpui::layer_shell::KeyboardInteractivity::None);
                window.resize(size(px(1.0), px(1.0)));
            });
        }
    }
}

impl Render for OsdView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgb(0x2e3440);
        let card_bg = rgb(0x3b4252);
        let frost0 = rgb(0xd8dee9);
        let muted_text = rgb(0x4c566a);
        let accent = rgb(0x88c0d0);
        let red = rgb(0xbf616a);
        let green = rgb(0xa3be8c);
        let yellow = rgb(0xebcb8b);

        let content = match &self.event {
            Some(OsdEvent::Volume { volume, muted }) => {
                let icon_name = if *muted {
                    "volume_off"
                } else if *volume == 0 {
                    "volume_mute"
                } else if *volume < 50 {
                    "volume_down"
                } else {
                    "volume_up"
                };
                let icon_color = if *muted { red } else { accent };
                let label = if *muted {
                    "Muted".to_string()
                } else {
                    format!("{}%", volume)
                };

                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .w_full()
                    .child(Icon::new(icon_name).size(px(22.0)).text_color(icon_color))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .flex_1()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(frost0).child("Volume"))
                                    .child(div().text_xs().text_color(icon_color).child(label)),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .h(px(6.0))
                                    .bg(card_bg)
                                    .rounded_full()
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .h_full()
                                            .w(relative(*volume as f32 / 100.0))
                                            .bg(if *muted { red } else { accent })
                                            .rounded_full(),
                                    ),
                            ),
                    )
            }

            Some(OsdEvent::Microphone { muted }) => {
                let icon_name = if *muted { "mic_off" } else { "mic" };
                let icon_color = if *muted { red } else { green };
                let label = if *muted { "Microphone Muted" } else { "Microphone Active" };

                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .w_full()
                    .child(Icon::new(icon_name).size(px(22.0)).text_color(icon_color))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(frost0).child("Microphone"))
                            .child(div().text_xs().text_color(icon_color).child(label)),
                    )
            }

            Some(OsdEvent::CapsLock { enabled }) => {
                let icon_color = if *enabled { yellow } else { muted_text };
                let label = if *enabled { "Caps Lock ON" } else { "Caps Lock OFF" };

                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .w_full()
                    .child(Icon::new("keyboard_capslock").size(px(22.0)).text_color(icon_color))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(frost0).child("Keyboard"))
                            .child(div().text_xs().font_weight(FontWeight::SEMIBOLD).text_color(icon_color).child(label)),
                    )
            }

            Some(OsdEvent::Clipboard { content }) => {
                let truncated = content.replace(['\n', '\r'], " ");
                let char_count = truncated.chars().count();
                let preview = if char_count > 32 {
                    format!("{}…", truncated.chars().take(32).collect::<String>())
                } else {
                    truncated
                };

                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .w_full()
                    .child(Icon::new("content_paste").size(px(22.0)).text_color(accent))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(frost0).child("Copied to Clipboard"))
                            .child(div().text_xs().text_color(muted_text).overflow_hidden().whitespace_nowrap().child(preview)),
                    )
            }

            None => div().child(div().text_xs().text_color(muted_text).child("OSD")),
        };

        div()
            .size_full()
            .flex()
            .flex_row()
            .child(
                div()
                    .h_full()
                    .w(px(CORNER_RADIUS))
                    .flex()
                    .flex_col()
                    .child(div().flex_1())
                    .child(
                        div().flex_none().child(
                            Corner::new(CornerPosition::BottomRight, px(CORNER_RADIUS)).color(bg),
                        ),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .size_full()
                    .bg(bg)
                    .rounded_t(px(CORNER_RADIUS))
                    .flex()
                    .items_center()
                    .px_3()
                    .child(content),
            )
            .child(
                div()
                    .h_full()
                    .w(px(CORNER_RADIUS))
                    .flex()
                    .flex_col()
                    .child(div().flex_1())
                    .child(
                        div().flex_none().child(
                            Corner::new(CornerPosition::BottomLeft, px(CORNER_RADIUS)).color(bg),
                        ),
                    ),
            )
    }
}
