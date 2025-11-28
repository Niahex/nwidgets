use crate::utils::icons;
use crate::services::pipewire::{AudioDevice, AudioStream, PipeWireService};
use gtk::prelude::*;
use gtk4 as gtk;

use super::section_helpers::{setup_expand_callback, setup_periodic_updates};

pub fn create_audio_section() -> (gtk::Box, gtk::Scale, gtk::Scale, gtk::Image, gtk::Image, gtk::Box, gtk::Button, gtk::Box, gtk::Button) {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 0);
    section.add_css_class("control-section");

    let title = gtk::Label::new(Some("Audio"));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    section.append(&title);

    // Volume controls with expand button
    let volume_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    volume_box.add_css_class("audio-volume-box");
    volume_box.set_hexpand(true);
    let volume_icon = icons::create_icon("sink-medium");
    volume_icon.add_css_class("control-icon");
    volume_icon.set_size_request(24, 24);
    let volume_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    volume_scale.add_css_class("control-scale");
    volume_scale.set_hexpand(true);
    volume_scale.set_draw_value(true);
    volume_scale.set_value_pos(gtk::PositionType::Right);

    // Connect volume change
    volume_scale.connect_value_changed(|scale| {
        let volume = scale.value() as u8;
        PipeWireService::set_volume(volume);
    });

    // Expand button for volume
    let volume_expand_btn = gtk::Button::from_icon_name("go-down-symbolic");
    volume_expand_btn.add_css_class("expand-button");
    volume_expand_btn.set_size_request(20, 35);

    volume_box.append(&volume_icon);
    volume_box.append(&volume_scale);
    volume_box.append(&volume_expand_btn);
    section.append(&volume_box);

    // Expanded box for volume details (initially hidden)
    let volume_expanded = create_volume_details();
    section.append(&volume_expanded);

    // Mic controls with expand button
    let mic_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    mic_box.add_css_class("audio-mic-box");
    mic_box.set_hexpand(true);
    let mic_icon = icons::create_icon("source-medium");
    mic_icon.add_css_class("control-icon");
    mic_icon.set_size_request(24, 24);
    let mic_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    mic_scale.add_css_class("control-scale");
    mic_scale.set_hexpand(true);
    mic_scale.set_draw_value(true);
    mic_scale.set_value_pos(gtk::PositionType::Right);

    // Connect mic volume change
    mic_scale.connect_value_changed(|scale| {
        let volume = scale.value() as u8;
        PipeWireService::set_mic_volume(volume);
    });

    // Expand button for mic
    let mic_expand_btn = gtk::Button::from_icon_name("go-down-symbolic");
    mic_expand_btn.add_css_class("expand-button");
    mic_expand_btn.set_size_request(20, 35);

    mic_box.append(&mic_icon);
    mic_box.append(&mic_scale);
    mic_box.append(&mic_expand_btn);
    section.append(&mic_box);

    // Expanded box for mic details (initially hidden)
    let mic_expanded = create_mic_details();
    section.append(&mic_expanded);

    (section, volume_scale, mic_scale, volume_icon, mic_icon, volume_expanded, volume_expand_btn, mic_expanded, mic_expand_btn)
}

pub fn setup_audio_section_callbacks(
    volume_expanded: &gtk::Box,
    volume_expand_btn: &gtk::Button,
    mic_expanded: &gtk::Box,
    mic_expand_btn: &gtk::Button,
    panels: &PanelManager,
) {
    setup_expand_callback(volume_expanded, volume_expand_btn, panels, "volume", populate_volume_details);
    setup_expand_callback(mic_expanded, mic_expand_btn, panels, "mic", populate_mic_details);
}

pub fn setup_audio_updates(
    volume_expanded: &gtk::Box,
    mic_expanded: &gtk::Box,
) {
    setup_periodic_updates(volume_expanded, 2, populate_volume_details);
    setup_periodic_updates(mic_expanded, 2, populate_mic_details);
}

// Struct pour gérer l'état des panneaux
#[derive(Clone)]
pub struct PanelManager {
    pub volume_expanded: gtk::Box,
    pub mic_expanded: gtk::Box,
    pub bluetooth_expanded: gtk::Box,
    pub network_expanded: gtk::Box,
}

impl PanelManager {
    pub fn new(
        volume_expanded: gtk::Box,
        mic_expanded: gtk::Box,
        bluetooth_expanded: gtk::Box,
        network_expanded: gtk::Box,
    ) -> Self {
        Self {
            volume_expanded,
            mic_expanded,
            bluetooth_expanded,
            network_expanded,
        }
    }

    pub fn collapse_all_except(&self, except: &str) {
        if except != "volume" {
            self.volume_expanded.set_visible(false);
        }
        if except != "mic" {
            self.mic_expanded.set_visible(false);
        }
        if except != "bluetooth" {
            self.bluetooth_expanded.set_visible(false);
        }
        if except != "network" {
            self.network_expanded.set_visible(false);
        }
    }
}

/// Determine the appropriate icon for a stream based on its title or app name
fn get_stream_icon(stream: &AudioStream) -> String {
    // First, try to use the app icon from PipeWire metadata
    if let Some(ref icon) = stream.app_icon {
        return icon.clone();
    }

    // Check window title first, then app name
    let search_text = stream
        .window_title
        .as_deref()
        .unwrap_or(&stream.app_name)
        .to_lowercase();

    // Check for specific platforms/apps
    if search_text.contains("youtube") {
        "youtube".to_string()
    } else if search_text.contains("twitch") {
        "twitch".to_string()
    } else if search_text.contains("discord") {
        "discord".to_string()
    } else if search_text.contains("firefox") {
        "firefox".to_string()
    } else if search_text.contains("chrome") || search_text.contains("chromium") {
        "chrome".to_string()
    } else if search_text.contains("vlc") {
        "vlc".to_string()
    } else if search_text.contains("spotify") {
        "spotify".to_string()
    } else {
        // Fallback to generic application icon
        "application-x-executable".to_string()
    }
}

/// Create the expanded audio details widget for output (sinks)
pub fn create_volume_details() -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    container.add_css_class("expanded-section");
    container.set_visible(false);

    container
}

/// Create the expanded audio details widget for input (sources)
pub fn create_mic_details() -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
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
    let sources = PipeWireService::list_sources();
    let device_dropdown = create_device_dropdown(sources, false);
    container.append(&device_dropdown);

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
    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    container.add_css_class("device-dropdown-container");

    let dropdown = gtk::DropDown::from_strings(
        &devices
            .iter()
            .map(|d| {
                if d.description.len() > 50 {
                    format!("{}...", &d.description[..47])
                } else {
                    d.description.clone()
                }
            })
            .collect::<Vec<_>>()
            .iter()
            .map(|s| s.as_str())
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
    let row = gtk::Box::new(gtk::Orientation::Vertical, 0);
    row.add_css_class("stream-row");

    // First line: icon, app name, and mute button
    let first_line = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    first_line.add_css_class("stream-first-line");

    // Determine icon based on window title or app name
    let icon_name = get_stream_icon(&stream);
    let app_icon = icons::create_icon(&icon_name);
    app_icon.add_css_class("stream-icon");
    app_icon.set_size_request(32, 32);
    first_line.append(&app_icon);

    // Display app name (from application.name)
    // Only use window_title if it's more descriptive than generic names
    let display_name = if let Some(ref title) = stream.window_title {
        let title_lower = title.to_lowercase();
        if title_lower == "audio stream"
            || title_lower == "playback"
            || title_lower == "record"
            || title_lower == "playstream"
            || title.is_empty()
        {
            &stream.app_name
        } else {
            title
        }
    } else {
        &stream.app_name
    };
    let app_label = gtk::Label::new(Some(display_name));
    app_label.add_css_class("stream-app-name");
    app_label.set_halign(gtk::Align::Start);
    app_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    first_line.append(&app_label);

    // Volume label
    let volume_label = gtk::Label::new(Some(&format!("{}%", stream.volume)));
    volume_label.add_css_class("stream-volume-label");
    volume_label.set_size_request(40, -1);
    first_line.append(&volume_label);

    // Mute button
    let mute_btn = gtk::Button::from_icon_name(if stream.muted {
        "audio-volume-muted-symbolic"
    } else {
        "audio-volume-high-symbolic"
    });
    mute_btn.add_css_class("mute-button");
    mute_btn.set_size_request(28, 28);
    mute_btn.set_valign(gtk::Align::Center);
    first_line.append(&mute_btn);

    row.append(&first_line);

    // Second line: volume slider only
    let volume_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    volume_scale.set_value(stream.volume as f64);
    volume_scale.add_css_class("stream-scale");
    volume_scale.set_size_request(150, -1);
    volume_scale.set_draw_value(false);

    row.append(&volume_scale);
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
