use crate::utils::{PinController, cef_manager::{CefManager, CefWebView}};
use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::Cell;
use std::rc::Rc;

pub struct ChatOverlay {
    pub window: gtk::ApplicationWindow,
    pub pin_controller: PinController,
    pub webview: CefWebView,
    #[allow(dead_code)]
    pub id: String,
    #[allow(dead_code)]
    is_pinned: Rc<Cell<bool>>,
}

const SITES: &[(&str, &str)] = &[
    ("Gemini", "https://gemini.google.com/"),
    ("DeepSeek", "https://chat.deepseek.com/"),
    ("AI Studio", "https://aistudio.google.com/apps"),
    (
        "DuckDuckGo AI",
        "https://duckduckgo.com/?q=DuckDuckGo+AI+Chat&ia=chat&duckai=1&atb=v495-1",
    ),
];

pub fn create_chat_overlay(application: &gtk::Application) -> ChatOverlay {
    // Initialize CEF
    let cef_manager = CefManager::instance();
    if let Ok(mut manager) = cef_manager.lock() {
        manager.initialize();
    }

    let mut webview = CefWebView::new();
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("Nwidgets Chat")
        .default_width(500)
        .default_height(600)
        .build();
    window.add_css_class("chat-window");

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Right, true);
    window.set_margin(Edge::Top, 10);
    window.set_margin(Edge::Right, 10);
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    let header_bar = gtk::HeaderBar::new();
    header_bar.add_css_class("chat-header");
    window.set_titlebar(Some(&header_bar));

    let site_dropdown = gtk::DropDown::from_strings(&SITES.iter().map(|(name, _)| *name).collect::<Vec<_>>());
    site_dropdown.set_selected(0);
    header_bar.pack_start(&site_dropdown);

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

    // Load initial site
    webview.load_url(SITES[0].1);

    // Site dropdown handler
    let webview_clone = CefWebView::new();
    site_dropdown.connect_selected_notify(move |dropdown| {
        let selected = dropdown.selected() as usize;
        if selected < SITES.len() {
            webview_clone.load_url(SITES[selected].1);
        }
    });

    // Close button handler
    let window_clone = window.clone();
    close_button.connect_clicked(move |_| {
        window_clone.set_visible(false);
    });

    // Window visibility toggle
    let window_clone = window.clone();
    let toggle_action = gtk4::gio::SimpleAction::new("toggle-chat", None);
    toggle_action.connect_activate(move |_, _| {
        if window_clone.is_visible() {
            window_clone.set_visible(false);
        } else {
            window_clone.present();
        }
    });
    application.add_action(&toggle_action);

    let id = uuid::Uuid::new_v4().to_string();

    ChatOverlay {
        window,
        pin_controller,
        webview,
        id,
        is_pinned,
    }
}
