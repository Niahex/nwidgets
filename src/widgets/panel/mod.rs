use gtk::prelude::*;
use gtk4 as gtk;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
pub mod modules; // Changed from `mod modules;` to `pub mod modules;`
use modules::active_window::ActiveWindowModule;
use modules::audio::{AudioDeviceType, AudioModule};
use modules::bluetooth::BluetoothModule;
use modules::datetime::DateTimeModule;
use modules::mpris::MprisModule;
use modules::network::NetworkModule;
use modules::pomodoro::PomodoroModule;
use modules::systray::SystrayModule;
use modules::workspaces::WorkspacesModule;

// Type aliases for clarity
type SinkModule = AudioModule;
type SourceModule = AudioModule;

pub fn create_panel_window(
    application: &gtk::Application,
) -> (
    gtk::ApplicationWindow,
    ActiveWindowModule,
    WorkspacesModule,
    MprisModule,
    BluetoothModule,
    NetworkModule,
    SystrayModule,
    SinkModule,
    SourceModule,
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
    window.set_exclusive_zone(50);

    let layout = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    layout.set_height_request(50);
    layout.set_hexpand(true);
    layout.add_css_class("panel");

    let left_section = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    left_section.add_css_class("panel-left");

    let active_window_module = ActiveWindowModule::new();
    left_section.append(&active_window_module.container);

    layout.append(&left_section);

    // Spacer pour centrer la section centrale
    let spacer_left = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer_left.set_hexpand(true);
    layout.append(&spacer_left);

    // Section centrale : CenterBox pour centrage absolu des workspaces
    let center_box = gtk::CenterBox::new();
    center_box.add_css_class("panel-center");

    // Pomodoro - aligné à droite, s'étend vers la gauche
    let pomodoro_module = PomodoroModule::new();
    pomodoro_module.container.set_halign(gtk::Align::End);
    pomodoro_module.container.set_hexpand(true);
    center_box.set_start_widget(Some(&pomodoro_module.container));

    // Workspaces au centre (position fixe)
    let workspaces_module = WorkspacesModule::new();
    workspaces_module.container.set_halign(gtk::Align::Center);
    workspaces_module.container.set_hexpand(false);
    center_box.set_center_widget(Some(&workspaces_module.container));

    // MPRIS - aligné à gauche, s'étend vers la droite
    let mpris_module = MprisModule::new();
    mpris_module.container.set_halign(gtk::Align::Start);
    mpris_module.container.set_hexpand(true);
    center_box.set_end_widget(Some(&mpris_module.container));

    layout.append(&center_box);

    // Spacer pour pousser la section droite à droite
    let spacer_right = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer_right.set_hexpand(true);
    layout.append(&spacer_right);

    // Section droite : Systray, puis les autres modules
    let right_section = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    right_section.add_css_class("panel-right");
    right_section.set_halign(gtk::Align::End);

    // Systray en premier (peut changer de taille)
    let systray_module = SystrayModule::new();
    right_section.append(&systray_module.container);

    // Autres modules alignés
    let mic_module = AudioModule::new(AudioDeviceType::Source);
    right_section.append(&mic_module.container);
    let volume_module = AudioModule::new(AudioDeviceType::Sink);
    right_section.append(&volume_module.container);
    let network_module = NetworkModule::new();
    right_section.append(&network_module.container);
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
        mpris_module,
        bluetooth_module,
        network_module,
        systray_module,
        volume_module,
        mic_module,
        pomodoro_module,
    )
}
