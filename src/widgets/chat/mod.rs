use gtk4 as gtk;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::Cell;
use std::env;
use std::fs;
use std::rc::Rc;
use webkit6::prelude::*;
use webkit6::{
    CookiePersistentStorage, HardwareAccelerationPolicy, Settings, UserMediaPermissionRequest,
    WebContext, WebView,
};

const SITES: &[(&str, &str)] = &[
    ("Gemini", "https://gemini.google.com/"),
    ("DeepSeek", "https://chat.deepseek.com/"),
    ("AI Studio", "https://aistudio.google.com/apps"),
    (
        "DuckDuckGo AI",
        "https://duckduckgo.com/?q=DuckDuckGo+AI+Chat&ia=chat&duckai=1&atb=v495-1",
    ),
];

const ICON_RESERVE_SPACE: &str = "󰐃"; // Icon for reserving space
const ICON_RELEASE_SPACE: &str = "󰐄"; // Icon for releasing space

pub fn create_chat_window(application: &gtk::Application) -> gtk::ApplicationWindow {
    // --- WebView Settings ---
    let settings = Settings::new();
    settings.set_javascript_can_access_clipboard(true);
    settings.set_enable_developer_extras(true);
    settings.set_user_agent(Some(
        "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/119.0",
    ));
    settings.set_hardware_acceleration_policy(HardwareAccelerationPolicy::Always);
    settings.set_enable_media_stream(true);
    settings.set_enable_webaudio(true);
    settings.set_enable_webrtc(true);
    settings.set_enable_mediasource(true);
    settings.set_enable_media(true);
    settings.set_enable_media_capabilities(true);
    settings.set_enable_encrypted_media(true);
    settings.set_enable_mock_capture_devices(true);
    settings.set_media_playback_requires_user_gesture(false);
    settings.set_media_playback_allows_inline(true);
    settings.set_media_content_types_requiring_hardware_support(None);

    // --- Create Context and WebView ---
    let context = WebContext::new();
    let webview = WebView::builder().web_context(&context).build();
    webview.set_settings(&settings);

    // --- Handle Permission Requests ---
    webview.connect_permission_request(|_webview, request| {
        if let Some(media_request) = request.downcast_ref::<UserMediaPermissionRequest>() {
            if media_request.is_for_audio_device() {
                println!("Microphone access requested. Granting permission.");
                request.allow();
                return true;
            }
        }
        println!("Permission request denied by default.");
        request.deny();
        false
    });

    // --- Persistent Cookie Setup ---
    if let Some(session) = webview.network_session() {
        if let Some(cookie_manager) = session.cookie_manager() {
            if let Some(home_dir) = env::home_dir() {
                let cookie_file = home_dir.join(".local/share/nwidgets/cookies.sqlite"); // Changed path
                if let Some(cookie_dir) = cookie_file.parent() {
                    if fs::create_dir_all(cookie_dir).is_ok() {
                        if let Some(path_str) = cookie_file.to_str() {
                            cookie_manager
                                .set_persistent_storage(path_str, CookiePersistentStorage::Sqlite);
                        }
                    }
                }
            }
        }
    }

    // Create a window and set its title
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("Nwidgets Chat") // Changed title
        .default_width(500)
        .default_height(600)
        .build();

    // --- GTK Layer Shell Setup ---
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_keyboard_mode(KeyboardMode::OnDemand);

    // --- Site Selector Dropdown ---
    let site_names: Vec<&str> = SITES.iter().map(|(name, _)| *name).collect();
    let model = gtk::StringList::new(&site_names);
    let dropdown = gtk::DropDown::new(Some(model), None::<gtk::Expression>);
    let webview_clone = webview.clone();
    dropdown.connect_selected_item_notify(move |dropdown| {
        if let Some(selected) = dropdown.selected_item() {
            if let Ok(pos) = selected.downcast::<gtk::StringObject>().map(|s| s.string()) {
                if let Some(p) = site_names.iter().position(|&name| name == pos) {
                    webview_clone.load_uri(SITES[p].1);
                }
            }
        }
    });

    // --- Toggle Button ---
    let toggle_button = gtk::Button::with_label(ICON_RESERVE_SPACE);
    let is_exclusive = Rc::new(Cell::new(false));
    let window_clone = window.clone();
    let toggle_button_clone = toggle_button.clone();
    toggle_button.connect_clicked(move |_| {
        if is_exclusive.get() {
            window_clone.set_exclusive_zone(0);
            is_exclusive.set(false);
            toggle_button_clone.set_label(ICON_RESERVE_SPACE);
        } else {
            window_clone.auto_exclusive_zone_enable();
            is_exclusive.set(true);
            toggle_button_clone.set_label(ICON_RELEASE_SPACE);
        }
    });

    // --- Layout ---
    let top_bar = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    top_bar.append(&dropdown);
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true); // Make the spacer expand horizontally
    top_bar.append(&spacer);
    top_bar.append(&toggle_button);

    let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
    layout.append(&top_bar);
    layout.append(&webview);
    webview.set_vexpand(true);

    // Set the layout as the child of the window
    window.set_child(Some(&layout));

    // Load the initial URL
    webview.load_uri(SITES[0].1);

    // Cacher la fenêtre par défaut au démarrage
    window.set_visible(false);

    // Ajouter l'action toggle-chat
    let toggle_action = gtk::gio::SimpleAction::new("toggle-chat", None);
    let window_clone = window.clone();
    toggle_action.connect_activate(move |_, _| {
        let is_visible = window_clone.is_visible();
        window_clone.set_visible(!is_visible);
        println!("[CHAT] Toggle chat window: {}", !is_visible);
    });

    application.add_action(&toggle_action);

    window
}
