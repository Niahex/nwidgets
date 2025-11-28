use gtk::prelude::*;
use gtk4 as gtk;

use super::network_details::{create_network_details, populate_network_details};
use super::audio_section::PanelManager;

pub fn create_network_section() -> (gtk::Box, gtk::Box, gtk::Button) {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 8);
    section.add_css_class("control-section");

    let title = gtk::Label::new(Some("Network"));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    section.append(&title);

    // Network info with expand button
    let network_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    // Network status label
    let network_label = gtk::Label::new(Some("VPN Connections"));
    network_label.set_halign(gtk::Align::Start);
    network_label.set_hexpand(true);
    network_box.append(&network_label);

    // Expand button for network
    let network_expand_btn = gtk::Button::from_icon_name("go-down-symbolic");
    network_expand_btn.add_css_class("expand-button");
    network_box.append(&network_expand_btn);

    section.append(&network_box);

    // Expanded box for network details (initially hidden)
    let network_expanded = create_network_details();
    section.append(&network_expanded);

    (section, network_expanded, network_expand_btn)
}

pub fn setup_network_section_callbacks(
    network_expanded: &gtk::Box,
    network_expand_btn: &gtk::Button,
    panels: &PanelManager,
) {
    // Setup mutual exclusion for network expand button
    let panels_clone = panels.clone();
    let network_expanded_clone = network_expanded.clone();
    network_expand_btn.connect_clicked(move |btn| {
        let is_visible = network_expanded_clone.is_visible();
        if !is_visible {
            panels_clone.collapse_all_except("network");
            network_expanded_clone.set_visible(true);
            btn.set_icon_name("go-up-symbolic");
            populate_network_details(&network_expanded_clone);
        } else {
            network_expanded_clone.set_visible(false);
            btn.set_icon_name("go-down-symbolic");
        }
    });
}

pub fn setup_network_updates(network_expanded: &gtk::Box) {
    let network_expanded_for_update = network_expanded.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        if network_expanded_for_update.is_visible() {
            populate_network_details(&network_expanded_for_update);
        }
        gtk::glib::ControlFlow::Continue
    });
}
