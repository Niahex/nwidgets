use crate::services::bluetooth::{BluetoothDevice, BluetoothService};
use gtk::prelude::*;
use gtk4 as gtk;

use super::audio_details::PanelManager;
use super::section_helpers::{setup_expand_callback, setup_periodic_updates};

pub fn create_bluetooth_section() -> (gtk::Box, gtk::Box, gtk::Button) {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 8);
    section.add_css_class("control-section");

    let title = gtk::Label::new(Some("Bluetooth"));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    section.append(&title);

    // Bluetooth toggle with expand button
    let bt_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let bt_toggle = gtk::Switch::new();
    bt_toggle.add_css_class("control-switch");
    bt_toggle.set_hexpand(true);

    // Expand button for bluetooth
    let bt_expand_btn = gtk::Button::from_icon_name("go-down-symbolic");
    bt_expand_btn.add_css_class("expand-button");

    bt_box.append(&bt_toggle);
    bt_box.append(&bt_expand_btn);
    section.append(&bt_box);

    // Expanded box for bluetooth details (initially hidden)
    let bt_expanded = create_bluetooth_details();
    section.append(&bt_expanded);

    (section, bt_expanded, bt_expand_btn)
}

pub fn setup_bluetooth_section_callbacks(
    bt_expanded: &gtk::Box,
    bt_expand_btn: &gtk::Button,
    panels: &PanelManager,
) {
    setup_expand_callback(bt_expanded, bt_expand_btn, panels, "bluetooth", populate_bluetooth_details);
}

pub fn setup_bluetooth_updates(bt_expanded: &gtk::Box) {
    setup_periodic_updates(bt_expanded, 2, populate_bluetooth_details);
}

/// Create the expanded bluetooth details widget
pub fn create_bluetooth_details() -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
    container.add_css_class("expanded-section");
    container.set_visible(false);

    container
}

/// Populate the bluetooth details section with devices
pub fn populate_bluetooth_details(container: &gtk::Box) {
    // Clear existing widgets
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    let devices = BluetoothService::list_devices();

    if devices.is_empty() {
        let empty_label = gtk::Label::new(Some("No Bluetooth devices found"));
        empty_label.add_css_class("empty-label");
        empty_label.set_halign(gtk::Align::Start);
        container.append(&empty_label);
    } else {
        for device in devices {
            let device_row = create_device_row(device);
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
