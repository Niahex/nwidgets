use gpui::*;
use gpui_component::Icon;
use nwidgets_component_system_tray::SystemTrayComponent;
use nwidgets_service_audio::{AudioService, AudioStateChanged};
use nwidgets_service_bluetooth::{BluetoothService, BluetoothStateChanged};
use nwidgets_service_network::{NetworkService, NetworkStateChanged};

pub struct QuickSettingsComponent {
    system_tray: Entity<SystemTrayComponent>,
    bluetooth: Entity<BluetoothService>,
    network: Entity<NetworkService>,
    audio: Entity<AudioService>,
}

impl QuickSettingsComponent {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let system_tray = cx.new(SystemTrayComponent::new);
        let bluetooth = BluetoothService::global(cx);
        let network = NetworkService::global(cx);
        let audio = AudioService::global(cx);

        cx.subscribe(&bluetooth, |_, _, _: &BluetoothStateChanged, cx| cx.notify()).detach();
        cx.subscribe(&network, |_, _, _: &NetworkStateChanged, cx| cx.notify()).detach();
        cx.subscribe(&audio, |_, _, _: &AudioStateChanged, cx| cx.notify()).detach();

        Self {
            system_tray,
            bluetooth,
            network,
            audio,
        }
    }
}

impl Render for QuickSettingsComponent {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let text_main = rgb(0xd8dee9);
        let text_muted = rgb(0x4c566a);
        let accent = rgb(0x88c0d0);
        let red = rgb(0xbf616a);
        let border_subtle = rgb(0x4c566a).opacity(0.6);

        let bt_state = self.bluetooth.read(cx).state.clone();
        let _net_state = self.network.read(cx).state.clone();
        let audio_state = self.audio.read(cx).state.clone();

        // 1. Bluetooth Icon: "bluetooth_connected", "bluetooth", or "bluetooth_disabled"
        let has_bt_device = bt_state.devices.iter().any(|d| d.connected);
        let (bt_icon_name, bt_icon_color) = if !bt_state.powered {
            ("bluetooth_disabled", text_muted)
        } else if has_bt_device {
            ("bluetooth_connected", accent)
        } else {
            ("bluetooth", text_main)
        };

        // 2. Network Icon: "lan"
        let (net_icon_name, net_icon_color) = ("lan", accent);

        // 3. Audio Sink Icon
        let (vol_icon_name, vol_icon_color) = if audio_state.sink_muted {
            ("volume_off", red)
        } else if audio_state.sink_volume == 0 {
            ("volume_mute", text_muted)
        } else if audio_state.sink_volume < 50 {
            ("volume_down", text_main)
        } else {
            ("volume_up", text_main)
        };

        // 4. Microphone Icon
        let (mic_icon_name, mic_icon_color) = if audio_state.source_muted {
            ("mic_off", red)
        } else {
            ("mic", text_main)
        };

        div()
            .flex()
            .items_center()
            .gap_4()
            .px_2()
            // ── SystemTray Component to the LEFT of Bluetooth ──
            .child(self.system_tray.clone())
            // Subtle vertical separator between SystemTray and QuickSettings controls
            .child(div().h(px(14.0)).w(px(1.0)).bg(border_subtle))
            .child(Icon::new(bt_icon_name).size(px(22.0)).text_color(bt_icon_color))
            .child(Icon::new(net_icon_name).size(px(22.0)).text_color(net_icon_color))
            .child(Icon::new(mic_icon_name).size(px(22.0)).text_color(mic_icon_color))
            .child(Icon::new(vol_icon_name).size(px(22.0)).text_color(vol_icon_color))
    }
}
