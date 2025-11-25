use crate::services::network::{NetworkService, VpnConnection};
use gtk::prelude::*;
use gtk4 as gtk;

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

    // VPN Connections section
    let vpn_label = gtk::Label::new(Some("VPN Connections"));
    vpn_label.add_css_class("subsection-title");
    vpn_label.set_halign(gtk::Align::Start);
    container.append(&vpn_label);

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
