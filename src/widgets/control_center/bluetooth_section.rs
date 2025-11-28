use gtk::prelude::*;
use gtk4 as gtk;

use super::bluetooth_details::{create_bluetooth_details, populate_bluetooth_details};
use super::audio_section::PanelManager;

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
    // Setup mutual exclusion for bluetooth expand button
    let panels_clone = panels.clone();
    let bt_expanded_clone = bt_expanded.clone();
    bt_expand_btn.connect_clicked(move |btn| {
        let is_visible = bt_expanded_clone.is_visible();
        if !is_visible {
            panels_clone.collapse_all_except("bluetooth");
            bt_expanded_clone.set_visible(true);
            btn.set_icon_name("go-up-symbolic");
            populate_bluetooth_details(&bt_expanded_clone);
        } else {
            bt_expanded_clone.set_visible(false);
            btn.set_icon_name("go-down-symbolic");
        }
    });
}

pub fn setup_bluetooth_updates(bt_expanded: &gtk::Box) {
    let bt_expanded_for_update = bt_expanded.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        if bt_expanded_for_update.is_visible() {
            populate_bluetooth_details(&bt_expanded_for_update);
        }
        gtk::glib::ControlFlow::Continue
    });
}
