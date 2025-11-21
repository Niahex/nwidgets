use gtk::prelude::*;
use gtk4 as gtk;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
pub mod modules; // Changed from `mod modules;` to `pub mod modules;`
use modules::active_window::ActiveWindowModule;
use modules::bluetooth::BluetoothModule;
use modules::datetime::DateTimeModule;
use modules::mic::MicModule;
use modules::pomodoro::PomodoroModule;
use modules::systray::SystrayModule;
use modules::volume::VolumeModule;
use modules::workspaces::WorkspacesModule;

pub fn create_panel_window(
    application: &gtk::Application,
) -> (
    gtk::ApplicationWindow,
    ActiveWindowModule,
    WorkspacesModule,
    BluetoothModule,
    SystrayModule,
    VolumeModule,
    MicModule,
    PomodoroModule,
) {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .build();

    // --- GTK Layer Shell Setup ---
    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_exclusive_zone(45);

    let layout = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    layout.set_height_request(45);
    layout.set_hexpand(true);

    // Section gauche : Active Window
    let active_window_module = ActiveWindowModule::new();
    layout.append(&active_window_module.container);

    // Spacer pour centrer les workspaces
    let spacer_left = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer_left.set_hexpand(true);
    layout.append(&spacer_left);

    // Section centrale : Workspaces
    let workspaces_module = WorkspacesModule::new();
    layout.append(&workspaces_module.container);

    // Spacer pour pousser les autres modules à droite
    let spacer_right = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer_right.set_hexpand(true);
    layout.append(&spacer_right);

    // Section droite : Systray, Pomodoro, Mic, Volume, Bluetooth, DateTime
    let right_section = gtk::Box::new(gtk::Orientation::Horizontal, 12); // gap-3
    let systray_module = SystrayModule::new();
    right_section.append(&systray_module.container);
    let pomodoro_module = PomodoroModule::new();
    right_section.append(&pomodoro_module.container);
    let mic_module = MicModule::new();
    right_section.append(&mic_module.container);
    let volume_module = VolumeModule::new();
    right_section.append(&volume_module.container);
    let bluetooth_module = BluetoothModule::new();
    right_section.append(&bluetooth_module.container);
    let datetime_module = DateTimeModule::new();
    right_section.append(&datetime_module.container);
    layout.append(&right_section);

    window.set_child(Some(&layout));

    (
        window,
        active_window_module,
        workspaces_module,
        bluetooth_module,
        systray_module,
        volume_module,
        mic_module,
        pomodoro_module,
    )
}
