use crate::icons;
use crate::services::pipewire::{AudioDevice, AudioStream, PipeWireService};
use gtk::prelude::*;
use gtk4 as gtk;

/// Create the expanded audio details widget for output (sinks)
pub fn create_volume_details() -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
    container.add_css_class("expanded-section");
    container.set_visible(false);

    container
}

/// Create the expanded audio details widget for input (sources)
pub fn create_mic_details() -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
    container.add_css_class("expanded-section");
    container.set_visible(false);

    container
}

/// Populate the volume details section with devices and applications
pub fn populate_volume_details(container: &gtk::Box) {
    // Clear existing widgets
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    // Output Devices section
    let devices_label = gtk::Label::new(Some("Output Devices"));
    devices_label.add_css_class("subsection-title");
    devices_label.set_halign(gtk::Align::Start);
    container.append(&devices_label);

    let sinks = PipeWireService::list_sinks();
    for sink in sinks {
        let device_row = create_device_row(sink, true);
        container.append(&device_row);
    }

    // Applications section
    let apps_label = gtk::Label::new(Some("Applications"));
    apps_label.add_css_class("subsection-title");
    apps_label.set_halign(gtk::Align::Start);
    apps_label.set_margin_top(12);
    container.append(&apps_label);

    let streams = PipeWireService::list_sink_inputs();
    if streams.is_empty() {
        let empty_label = gtk::Label::new(Some("No active playback"));
        empty_label.add_css_class("empty-label");
        empty_label.set_halign(gtk::Align::Start);
        container.append(&empty_label);
    } else {
        for stream in streams {
            let stream_row = create_stream_row(stream);
            container.append(&stream_row);
        }
    }
}

/// Populate the mic details section with devices and applications
pub fn populate_mic_details(container: &gtk::Box) {
    // Clear existing widgets
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    // Input Devices section
    let devices_label = gtk::Label::new(Some("Input Devices"));
    devices_label.add_css_class("subsection-title");
    devices_label.set_halign(gtk::Align::Start);
    container.append(&devices_label);

    let sources = PipeWireService::list_sources();
    for source in sources {
        let device_row = create_device_row(source, false);
        container.append(&device_row);
    }

    // Recording Applications section
    let apps_label = gtk::Label::new(Some("Recording Applications"));
    apps_label.add_css_class("subsection-title");
    apps_label.set_halign(gtk::Align::Start);
    apps_label.set_margin_top(12);
    container.append(&apps_label);

    let streams = PipeWireService::list_source_outputs();
    if streams.is_empty() {
        let empty_label = gtk::Label::new(Some("No active recording"));
        empty_label.add_css_class("empty-label");
        empty_label.set_halign(gtk::Align::Start);
        container.append(&empty_label);
    } else {
        for stream in streams {
            let stream_row = create_stream_row(stream);
            container.append(&stream_row);
        }
    }
}

fn create_device_row(device: AudioDevice, is_sink: bool) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("device-row");
    row.set_margin_start(8);
    row.set_margin_top(4);
    row.set_margin_bottom(4);

    // Radio button to indicate if it's the default device
    let radio = gtk::CheckButton::new();
    radio.set_active(device.is_default);
    radio.add_css_class("device-radio");

    // Device name
    let label = gtk::Label::new(Some(&device.description));
    label.set_halign(gtk::Align::Start);
    label.set_hexpand(true);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);

    row.append(&radio);
    row.append(&label);

    // Connect radio button to set default device
    let device_id = device.id;
    radio.connect_toggled(move |btn| {
        if btn.is_active() {
            if is_sink {
                PipeWireService::set_default_sink(device_id);
            } else {
                PipeWireService::set_default_source(device_id);
            }
        }
    });

    row
}

fn create_stream_row(stream: AudioStream) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 4);
    row.add_css_class("stream-row");
    row.set_margin_start(8);
    row.set_margin_top(4);
    row.set_margin_bottom(4);

    // App name header
    let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let app_label = gtk::Label::new(Some(&stream.app_name));
    app_label.add_css_class("stream-app-name");
    app_label.set_halign(gtk::Align::Start);
    app_label.set_hexpand(true);

    // Mute button
    let mute_btn = gtk::Button::from_icon_name(if stream.muted {
        "audio-volume-muted-symbolic"
    } else {
        "audio-volume-high-symbolic"
    });
    mute_btn.add_css_class("mute-button");

    header_box.append(&app_label);
    header_box.append(&mute_btn);

    // Volume slider
    let volume_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let volume_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    volume_scale.set_value(stream.volume as f64);
    volume_scale.set_hexpand(true);
    volume_scale.add_css_class("stream-scale");
    volume_scale.set_draw_value(true);
    volume_scale.set_value_pos(gtk::PositionType::Right);

    volume_box.append(&volume_scale);

    row.append(&header_box);
    row.append(&volume_box);

    // Connect volume change
    let stream_id = stream.id;
    volume_scale.connect_value_changed(move |scale| {
        let volume = scale.value() as u8;
        PipeWireService::set_stream_volume(stream_id, volume);
    });

    // Connect mute toggle
    let stream_id = stream.id;
    mute_btn.connect_clicked(move |_| {
        PipeWireService::toggle_stream_mute(stream_id);
    });

    row
}
