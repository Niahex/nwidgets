use crate::services::network::{NetworkService, NetworkState, VpnConnection};
use gtk::prelude::*;
use gtk4 as gtk;

/// Update the network details section with data from NetworkState
pub fn update_network_details(container: &gtk::Box, state: &NetworkState) {
    // Clear existing widgets
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    // VPN Section Title
    let vpn_title = gtk::Label::new(Some("VPN Connections"));
    vpn_title.add_css_class("subsection-title");
    vpn_title.set_halign(gtk::Align::Start);
    container.append(&vpn_title);

    if state.vpn_connections.is_empty() {
        let empty_label = gtk::Label::new(Some("No VPN connections configured"));
        empty_label.add_css_class("empty-label");
        empty_label.set_halign(gtk::Align::Start);
        container.append(&empty_label);
    } else {
        for vpn in &state.vpn_connections {
            let vpn_row = create_vpn_row(vpn.clone());
            container.append(&vpn_row);
        }
    }
}

fn create_vpn_row(vpn: VpnConnection) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("vpn-connection-row");
    row.set_margin_start(8);
    row.set_margin_top(4);
    row.set_margin_bottom(4);

    let info_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    info_box.set_hexpand(true);

    let name_label = gtk::Label::new(Some(&vpn.name));
    name_label.add_css_class("vpn-name");
    name_label.set_halign(gtk::Align::Start);
    info_box.append(&name_label);

    let type_label = gtk::Label::new(Some(&vpn.vpn_type));
    type_label.add_css_class("vpn-type");
    type_label.set_halign(gtk::Align::Start);
    info_box.append(&type_label);

    row.append(&info_box);

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

/// Legacy function for compatibility (will be removed)
pub fn populate_network_details(container: &gtk::Box) {
    // This is now just a wrapper that fetches state once
    let state = crate::utils::runtime::block_on(NetworkService::get_network_state()).unwrap_or(NetworkState {
        connected: false,
        connection_type: crate::services::network::ConnectionType::None,
        signal_strength: 0,
        ssid: None,
        vpn_active: false,
        vpn_connections: Vec::new(),
    });
    update_network_details(container, &state);
}