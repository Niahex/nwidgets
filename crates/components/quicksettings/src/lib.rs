use gpui::*;
use gpui_component::Icon;

pub struct QuickSettingsComponent {
    pub wifi_enabled: bool,
    pub bluetooth_enabled: bool,
    pub volume_level: u32,
    pub mic_level: u32,
}

impl QuickSettingsComponent {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            wifi_enabled: true,
            bluetooth_enabled: true,
            volume_level: 80,
            mic_level: 100,
        }
    }
}

impl Render for QuickSettingsComponent {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let text_main = rgb(0xd8dee9);
        let text_muted = rgb(0x4c566a);

        div()
            .flex()
            .items_center()
            .gap_4()
            .px_2()
            // Bluetooth Icon
            .child(
                Icon::new(if self.bluetooth_enabled { "bluetooth" } else { "bluetooth_disabled" })
                    .size(px(30.0))
                    .text_color(if self.bluetooth_enabled { text_main } else { text_muted }),
            )
            // Wi-Fi Icon
            .child(
                Icon::new(if self.wifi_enabled { "wifi" } else { "wifi_off" })
                    .size(px(30.0))
                    .text_color(if self.wifi_enabled { text_main } else { text_muted }),
            )
            // Mic Icon
            .child(
                Icon::new("mic")
                    .size(px(30.0))
                    .text_color(text_main),
            )
            // Volume Icon
            .child(
                Icon::new("volume_up")
                    .size(px(30.0))
                    .text_color(text_main),
            )
    }
}
