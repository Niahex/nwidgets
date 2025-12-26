use crate::services::bluetooth::{BluetoothDevice, BluetoothService, BluetoothState};
use gtk::prelude::*;
use gtk4 as gtk;

/// Update the bluetooth details section with data from BluetoothState
pub fn update_bluetooth_details(container: &gtk::Box, state: &BluetoothState) {
    // Clear existing widgets
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    if state.devices.is_empty() {
        let empty_label = gtk::Label::new(Some("No Bluetooth devices found"));
        empty_label.add_css_class("empty-label");
        empty_label.set_halign(gtk::Align::Start);
        container.append(&empty_label);
    } else {
        for device in &state.devices {
            let device_row = create_device_row(device.clone());
            container.append(&device_row);
        }
    }
}

fn create_device_row(device: BluetoothDevice) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("bluetooth-device-row");
    row.set_margin_start(8);
    row.set_margin_top(4);
    row.set_margin_bottom(4);

    // Device info box
    let info_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    info_box.set_hexpand(true);

    // Device name
    let name_label = gtk::Label::new(Some(&device.name));
    name_label.add_css_class("bluetooth-device-name");
    name_label.set_halign(gtk::Align::Start);
    info_box.append(&name_label);

    // Device address (smaller text)
    let address_label = gtk::Label::new(Some(&device.address));
    address_label.add_css_class("bluetooth-device-address");
    address_label.set_halign(gtk::Align::Start);
    info_box.append(&address_label);

    row.append(&info_box);

    // Status indicator (only if paired but not connected)
    if !device.connected && device.paired {
        let paired_label = gtk::Label::new(Some("Paired"));
        paired_label.add_css_class("bluetooth-paired");
        row.append(&paired_label);
    }

    // Toggle switch for connect/disconnect
    let toggle = gtk::Switch::new();
    toggle.set_active(device.connected);
    toggle.add_css_class("bluetooth-toggle");

    let device_path = device.path.clone();
    toggle.connect_state_set(move |_, state| {
        if state {
            BluetoothService::connect_device(&device_path);
        } else {
            BluetoothService::disconnect_device(&device_path);
        }
        gtk::glib::Propagation::Proceed
    });

    row.append(&toggle);

    row
}