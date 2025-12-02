use crate::services::network::{NetworkService, VpnConnection};
use gtk::prelude::*;
use gtk4 as gtk;

use super::audio_details::PanelManager;
use super::section_helpers::{setup_expand_callback, setup_periodic_updates};

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
    setup_expand_callback(
        network_expanded,
        network_expand_btn,
        panels,
        "network",
        populate_network_details,
    );
}

pub fn setup_network_updates(network_expanded: &gtk::Box) {
    setup_periodic_updates(network_expanded, 2, populate_network_details);
}

/// Create the expanded network/VPN details widget
pub fn create_network_details() -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
    container.add_css_class("expanded-section");
    container.set_visible(false);

    container
}

/// Populate the network details section with VPN connections
pub fn populate_network_details(container: &gtk::Box) {
    // Clear existing widgets
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    let vpn_connections = NetworkService::list_vpn_connections();

    if vpn_connections.is_empty() {
        let empty_label = gtk::Label::new(Some("No VPN connections configured"));
        empty_label.add_css_class("empty-label");
        empty_label.set_halign(gtk::Align::Start);
        container.append(&empty_label);
    } else {
        for vpn in vpn_connections {
            let vpn_row = create_vpn_row(vpn);
            container.append(&vpn_row);
        }
    }
}

fn create_vpn_row(vpn: VpnConnection) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("vpn-row");
    row.set_margin_start(8);
    row.set_margin_top(4);
    row.set_margin_bottom(4);

    // VPN info box
    let info_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    info_box.set_hexpand(true);

    // VPN name
    let name_label = gtk::Label::new(Some(&vpn.name));
    name_label.add_css_class("vpn-name");
    name_label.set_halign(gtk::Align::Start);
    info_box.append(&name_label);

    // VPN type (smaller text)
    let type_label = gtk::Label::new(Some(&format!("Type: {}", vpn.vpn_type)));
    type_label.add_css_class("vpn-type");
    type_label.set_halign(gtk::Align::Start);
    info_box.append(&type_label);

    row.append(&info_box);

    // Toggle switch for connect/disconnect
    let toggle = gtk::Switch::new();
    toggle.set_active(vpn.active);
    toggle.add_css_class("vpn-toggle");

    let vpn_path = vpn.path.clone();
    toggle.connect_state_set(move |_, state| {
        if state {
            NetworkService::connect_vpn(&vpn_path);
        } else {
            NetworkService::disconnect_vpn(&vpn_path);
        }
        gtk::glib::Propagation::Proceed
    });

    row.append(&toggle);

    row
}
