use crate::utils::icons;
use crate::services::pipewire::{AudioState, PipeWireService};
use gtk::prelude::*;
use gtk4 as gtk;

use super::audio_details::{create_volume_details, create_mic_details, populate_volume_details, populate_mic_details};

pub fn create_audio_section() -> (gtk::Box, gtk::Scale, gtk::Scale, gtk::Image, gtk::Image, gtk::Box, gtk::Button, gtk::Box, gtk::Button) {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 8);
    section.add_css_class("control-section");

    let title = gtk::Label::new(Some("Audio"));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    section.append(&title);

    // Volume controls with expand button
    let volume_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let volume_icon = icons::create_icon("sink-medium");
    volume_icon.add_css_class("control-icon");
    let volume_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    volume_scale.set_hexpand(true);
    volume_scale.add_css_class("control-scale");
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

    volume_box.append(&volume_icon);
    volume_box.append(&volume_scale);
    volume_box.append(&volume_expand_btn);
    section.append(&volume_box);

    // Expanded box for volume details (initially hidden)
    let volume_expanded = create_volume_details();
    section.append(&volume_expanded);

    // Mic controls with expand button
    let mic_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let mic_icon = icons::create_icon("source-medium");
    mic_icon.add_css_class("control-icon");
    let mic_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    mic_scale.set_hexpand(true);
    mic_scale.add_css_class("control-scale");
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
    // Setup mutual exclusion for volume expand button
    let panels_clone = panels.clone();
    let volume_expanded_clone = volume_expanded.clone();
    volume_expand_btn.connect_clicked(move |btn| {
        let is_visible = volume_expanded_clone.is_visible();
        if !is_visible {
            panels_clone.collapse_all_except("volume");
            volume_expanded_clone.set_visible(true);
            btn.set_icon_name("go-up-symbolic");
            populate_volume_details(&volume_expanded_clone);
        } else {
            volume_expanded_clone.set_visible(false);
            btn.set_icon_name("go-down-symbolic");
        }
    });

    // Setup mutual exclusion for mic expand button
    let panels_clone = panels.clone();
    let mic_expanded_clone = mic_expanded.clone();
    mic_expand_btn.connect_clicked(move |btn| {
        let is_visible = mic_expanded_clone.is_visible();
        if !is_visible {
            panels_clone.collapse_all_except("mic");
            mic_expanded_clone.set_visible(true);
            btn.set_icon_name("go-up-symbolic");
            populate_mic_details(&mic_expanded_clone);
        } else {
            mic_expanded_clone.set_visible(false);
            btn.set_icon_name("go-down-symbolic");
        }
    });
}

pub fn setup_audio_updates(
    volume_expanded: &gtk::Box,
    mic_expanded: &gtk::Box,
) {
    // Setup periodic updates (only update if visible)
    let volume_expanded_for_update = volume_expanded.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        if volume_expanded_for_update.is_visible() {
            populate_volume_details(&volume_expanded_for_update);
        }
        gtk::glib::ControlFlow::Continue
    });

    let mic_expanded_for_update = mic_expanded.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        if mic_expanded_for_update.is_visible() {
            populate_mic_details(&mic_expanded_for_update);
        }
        gtk::glib::ControlFlow::Continue
    });
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
