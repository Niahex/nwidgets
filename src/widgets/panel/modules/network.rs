use crate::icons;
use crate::services::network::NetworkState;
use gtk::prelude::*;
use gtk4 as gtk;

#[derive(Clone)]
pub struct NetworkModule {
    pub container: gtk::CenterBox,
    icon: gtk::Image,
}

impl NetworkModule {
    pub fn new() -> Self {
        let container = gtk::CenterBox::new();
        container.add_css_class("network-widget");
        container.set_width_request(35);
        container.set_height_request(50);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let icon = icons::create_icon_with_size("network-disconnected", Some(20));
        icon.add_css_class("network-icon");
        icon.set_halign(gtk::Align::Center);
        icon.set_valign(gtk::Align::Center);

        container.set_center_widget(Some(&icon));

        // Click handler to open control center with network section
        let gesture = gtk::GestureClick::new();
        gesture.connect_released(move |_, _, _, _| {
            if let Some(app) = gtk::gio::Application::default() {
                if let Some(action) = app.lookup_action("toggle-control-center") {
                    action.activate(Some(&"network".to_variant()));
                }
            }
        });
        container.add_controller(gesture);

        Self { container, icon }
    }

    pub fn update(&self, state: NetworkState) {
        let icon_name = state.get_icon_name();

        if let Some(paintable) = icons::get_paintable_with_size(icon_name, Some(20)) {
            self.icon.set_paintable(Some(&paintable));
        }
    }
}
