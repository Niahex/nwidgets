use crate::utils::PinController;
use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
pub struct JisigOverlay {
    pub window: gtk::ApplicationWindow,
    pub pin_controller: PinController,
    #[allow(dead_code)]
    pub id: String,
    #[allow(dead_code)]
    is_pinned: Rc<Cell<bool>>,
}

const DEFAULT_URL: &str = "http://127.0.0.1:3000/private";

pub fn create_jisig_overlay(application: &gtk::Application) -> JisigOverlay {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("Nwidgets Jisig")
        .default_width(800)
        .default_height(600)
        .build();
    window.add_css_class("jisig-window");

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_margin(Edge::Top, 10);
    window.set_margin(Edge::Left, 10);

    let header_bar = gtk::HeaderBar::new();
    header_bar.add_css_class("jisig-header");
    window.set_titlebar(Some(&header_bar));

    // Create pin controller with required parameters
    let is_pinned = Rc::new(Cell::new(false));
    let pin_icon = gtk::Image::from_icon_name("view-pin-symbolic");
    let pin_controller = PinController::new(window.clone(), is_pinned.clone(), pin_icon);
    let pin_button = gtk::Button::new();
    pin_button.set_child(Some(&gtk::Image::from_icon_name("view-pin-symbolic")));
    header_bar.pack_end(&pin_button);

    let close_button = gtk::Button::from_icon_name("window-close-symbolic");
    close_button.add_css_class("destructive-action");
    header_bar.pack_end(&close_button);

    let reload_button = gtk::Button::from_icon_name("view-refresh-symbolic");
    reload_button.set_tooltip_text(Some("Reload"));
    header_bar.pack_start(&reload_button);

    // For now, use a placeholder label instead of CEF webview
    let placeholder = gtk::Label::new(Some(&format!("CEF WebView will load: {}", DEFAULT_URL)));
    placeholder.add_css_class("jisig-placeholder");
    window.set_child(Some(&placeholder));

    // Close button handler
    let window_clone = window.clone();
    close_button.connect_clicked(move |_| {
        window_clone.set_visible(false);
    });

    // Window visibility toggle
    let window_clone = window.clone();
    let toggle_action = gtk4::gio::SimpleAction::new("toggle-jisig", None);
    toggle_action.connect_activate(move |_, _| {
        if window_clone.is_visible() {
            window_clone.set_visible(false);
        } else {
            window_clone.present();
        }
    });
    application.add_action(&toggle_action);

    let id = uuid::Uuid::new_v4().to_string();

    JisigOverlay {
        window,
        pin_controller,
        id,
        is_pinned,
    }
}
