use crate::icons;
use gtk::prelude::*;
use gtk4 as gtk;

/// Configuration for creating a panel module
pub struct PanelModuleConfig {
    /// CSS class for the container widget (e.g., "bluetooth-widget")
    pub widget_class: &'static str,
    /// CSS class for the icon (e.g., "bluetooth-icon")
    pub icon_class: &'static str,
    /// Initial icon name
    pub initial_icon: &'static str,
    /// Control center section to open on click (e.g., "bluetooth")
    pub control_center_section: &'static str,
    /// Width request in pixels (default: 35)
    pub width: i32,
    /// Height request in pixels (default: 50)
    pub height: i32,
    /// Icon size in pixels (default: 20)
    pub icon_size: u32,
}

impl Default for PanelModuleConfig {
    fn default() -> Self {
        Self {
            widget_class: "panel-module",
            icon_class: "panel-icon",
            initial_icon: "default",
            control_center_section: "",
            width: 35,
            height: 50,
            icon_size: 20,
        }
    }
}

impl PanelModuleConfig {
    /// Create a new config with the specified classes and icon
    pub fn new(
        widget_class: &'static str,
        icon_class: &'static str,
        initial_icon: &'static str,
        control_center_section: &'static str,
    ) -> Self {
        Self {
            widget_class,
            icon_class,
            initial_icon,
            control_center_section,
            ..Default::default()
        }
    }

    /// Build a panel module with this configuration
    pub fn build(self) -> (gtk::CenterBox, gtk::Image) {
        let container = gtk::CenterBox::new();
        container.add_css_class(self.widget_class);
        container.set_width_request(self.width);
        container.set_height_request(self.height);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let icon = icons::create_icon_with_size(self.initial_icon, Some(self.icon_size));
        icon.add_css_class(self.icon_class);
        icon.set_halign(gtk::Align::Center);
        icon.set_valign(gtk::Align::Center);

        container.set_center_widget(Some(&icon));

        // Add click handler if control center section is specified
        if !self.control_center_section.is_empty() {
            add_control_center_click_handler(&container, self.control_center_section);
        }

        (container, icon)
    }
}

/// Helper function to update a panel module icon
pub fn update_icon(icon: &gtk::Image, icon_name: &str, size: Option<u32>) {
    if let Some(paintable) = icons::get_paintable_with_size(icon_name, size) {
        icon.set_paintable(Some(&paintable));
    }
}

/// Add a click handler that opens the control center with a specific section
pub fn add_control_center_click_handler(widget: &impl gtk::prelude::WidgetExt, section: &str) {
    let section = section.to_string();
    let gesture = gtk::GestureClick::new();
    gesture.connect_released(move |_, _, _, _| {
        if let Some(app) = gtk::gio::Application::default() {
            if let Some(action) = app.lookup_action("toggle-control-center") {
                action.activate(Some(&section.to_variant()));
            }
        }
    });
    widget.add_controller(gesture);
}

/// Macro to reduce boilerplate for simple panel modules
#[macro_export]
macro_rules! panel_module {
    (
        $module_name:ident,
        $state_type:ty,
        widget_class: $widget_class:expr,
        icon_class: $icon_class:expr,
        initial_icon: $initial_icon:expr,
        section: $section:expr,
        update_icon: $update_fn:expr
    ) => {
        #[derive(Clone)]
        pub struct $module_name {
            pub container: gtk::CenterBox,
            icon: gtk::Image,
        }

        impl $module_name {
            pub fn new() -> Self {
                let config = $crate::widgets::panel::modules::base::PanelModuleConfig::new(
                    $widget_class,
                    $icon_class,
                    $initial_icon,
                    $section,
                );
                let (container, icon) = config.build();
                Self { container, icon }
            }

            pub fn update(&self, state: $state_type) {
                let icon_name = $update_fn(state);
                $crate::widgets::panel::modules::base::update_icon(&self.icon, icon_name, Some(20));
            }
        }
    };
}
