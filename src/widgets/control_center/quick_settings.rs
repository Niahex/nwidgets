use gtk::prelude::*;
use gtk4 as gtk;

pub fn create_quick_settings_section() -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 8);
    section.add_css_class("control-section");

    let title = gtk::Label::new(Some("Quick Settings"));
    title.add_css_class("section-title");
    title.set_halign(gtk::Align::Start);
    section.append(&title);

    // Grid for quick toggle buttons
    let grid = gtk::Grid::new();
    grid.set_row_spacing(8);
    grid.set_column_spacing(8);
    grid.set_column_homogeneous(true);

    // WiFi toggle
    let wifi_button = create_quick_toggle_button("WiFi", "network-wifi-high", true);
    grid.attach(&wifi_button, 0, 0, 1, 1);

    // Bluetooth toggle
    let bluetooth_button = create_quick_toggle_button("Bluetooth", "bluetooth", false);
    grid.attach(&bluetooth_button, 1, 0, 1, 1);

    // Do Not Disturb toggle
    let dnd_button = create_quick_toggle_button("Do Not Disturb", "notification", false);
    grid.attach(&dnd_button, 0, 1, 1, 1);

    // Night Light toggle
    let night_light_button = create_quick_toggle_button("Night Light", "weather-clear-night", false);
    grid.attach(&night_light_button, 1, 1, 1, 1);

    section.append(&grid);
    section
}

fn create_quick_toggle_button(label: &str, icon_name: &str, active: bool) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("quick-toggle");
    if active {
        button.add_css_class("active");
    }

    let content = gtk::Box::new(gtk::Orientation::Vertical, 4);
    
    let icon = gtk::Image::from_icon_name(icon_name);
    icon.add_css_class("quick-toggle-icon");
    
    let label_widget = gtk::Label::new(Some(label));
    label_widget.add_css_class("quick-toggle-label");
    
    content.append(&icon);
    content.append(&label_widget);
    button.set_child(Some(&content));

    // Connect toggle behavior
    button.connect_clicked(|btn| {
        if btn.has_css_class("active") {
            btn.remove_css_class("active");
        } else {
            btn.add_css_class("active");
        }
    });

    button
}
