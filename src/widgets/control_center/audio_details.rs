use crate::icons;
use crate::services::pipewire::{AudioDevice, AudioStream, PipeWireService};
use gtk::prelude::*;
use gtk4 as gtk;

/// Determine the appropriate icon for a stream based on its title or app name
fn get_stream_icon(stream: &AudioStream) -> &'static str {
    // Check window title first, then app name
    let search_text = stream
        .window_title
        .as_deref()
        .unwrap_or(&stream.app_name)
        .to_lowercase();

    // Check for specific platforms/apps
    if search_text.contains("youtube") {
        "youtube"
    } else if search_text.contains("twitch") {
        "twitch"
    } else if search_text.contains("spotify") {
        "spotify"
    } else if search_text.contains("discord") {
        "discord"
    } else if search_text.contains("firefox") {
        "firefox"
    } else if search_text.contains("chrome") || search_text.contains("chromium") {
        "chrome"
    } else if search_text.contains("vlc") {
        "vlc"
    } else if search_text.contains("mpv") {
        "mpv"
    } else {
        // Try to use the app icon from metadata if available
        // Otherwise fallback to generic application icon
        "application-x-executable"
    }
}

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

    let sinks = PipeWireService::list_sinks();
    let device_dropdown = create_device_dropdown(sinks, true);
    container.append(&device_dropdown);

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
    let device_dropdown = create_device_dropdown(sources, false);
    container.append(&device_dropdown);

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

fn create_device_dropdown(devices: Vec<AudioDevice>, is_sink: bool) -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
    container.set_margin_start(8);
    container.set_margin_top(4);
    container.set_margin_bottom(8);

    let dropdown = gtk::DropDown::from_strings(
        &devices
            .iter()
            .map(|d| d.description.as_str())
            .collect::<Vec<_>>(),
    );
    dropdown.add_css_class("device-dropdown");

    // Find and set the default device as selected
    if let Some(default_idx) = devices.iter().position(|d| d.is_default) {
        dropdown.set_selected(default_idx as u32);
    }

    // Clone devices for the closure
    let devices_clone = devices.clone();
    dropdown.connect_selected_notify(move |dropdown| {
        let selected_idx = dropdown.selected() as usize;
        if let Some(device) = devices_clone.get(selected_idx) {
            if is_sink {
                PipeWireService::set_default_sink(device.id);
            } else {
                PipeWireService::set_default_source(device.id);
            }
        }
    });

    container.append(&dropdown);
    container
}

fn create_stream_row(stream: AudioStream) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("stream-row");
    row.set_margin_start(8);
    row.set_margin_top(4);
    row.set_margin_bottom(4);

    // Determine icon based on window title or app name
    let icon_name = get_stream_icon(&stream);
    let app_icon = icons::create_icon(icon_name);
    app_icon.set_pixel_size(32);
    app_icon.add_css_class("stream-icon");
    row.append(&app_icon);

    // Content box with app name and volume control
    let content_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    content_box.set_hexpand(true);

    // Display window title if available, otherwise app name
    let display_name = stream.window_title.as_deref().unwrap_or(&stream.app_name);
    let app_label = gtk::Label::new(Some(display_name));
    app_label.add_css_class("stream-app-name");
    app_label.set_halign(gtk::Align::Start);
    app_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    content_box.append(&app_label);

    // Volume slider with label
    let volume_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let volume_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    volume_scale.set_value(stream.volume as f64);
    volume_scale.set_hexpand(true);
    volume_scale.add_css_class("stream-scale");
    volume_scale.set_draw_value(false);

    // Volume label
    let volume_label = gtk::Label::new(Some(&format!("{}%", stream.volume)));
    volume_label.add_css_class("stream-volume-label");
    volume_label.set_width_chars(4);

    volume_box.append(&volume_scale);
    volume_box.append(&volume_label);
    content_box.append(&volume_box);

    row.append(&content_box);

    // Mute button
    let mute_btn = gtk::Button::from_icon_name(if stream.muted {
        "audio-volume-muted-symbolic"
    } else {
        "audio-volume-high-symbolic"
    });
    mute_btn.add_css_class("mute-button");
    mute_btn.set_valign(gtk::Align::Center);
    row.append(&mute_btn);

    // Connect volume change
    let stream_id = stream.id;
    let volume_label_clone = volume_label.clone();
    volume_scale.connect_value_changed(move |scale| {
        let volume = scale.value() as u8;
        volume_label_clone.set_text(&format!("{}%", volume));
        PipeWireService::set_stream_volume(stream_id, volume);
    });

    // Connect mute toggle
    let stream_id = stream.id;
    mute_btn.connect_clicked(move |_| {
        PipeWireService::toggle_stream_mute(stream_id);
    });

    row
}
