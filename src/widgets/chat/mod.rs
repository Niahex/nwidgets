use gtk4 as gtk;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::{Cell, RefCell};
use std::env;
use std::fs;
use std::rc::Rc;
use webkit6::prelude::*;
use webkit6::{
    CookiePersistentStorage, HardwareAccelerationPolicy, Settings, UserMediaPermissionRequest,
    WebContext, WebView,
};
use crate::services::PinController;

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

pub fn create_chat_window(application: &gtk::Application) -> (gtk::ApplicationWindow, PinController) {
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
    webview.add_css_class("chat-webview");


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
    window.add_css_class("chat-window");

    // --- GTK Layer Shell Setup ---
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    // --- Site Selector Dropdown ---
    struct SiteData {
        name: &'static str,
        url: &'static str,
        icon_path: String, // Path to the SVG icon
    }

    let sites_with_icons: Rc<Vec<SiteData>> = Rc::new(SITES.iter().map(|(name, url)| {
        let icon_filename = format!("{}.svg", name.to_lowercase().replace(" ", "-")); // e.g., "gemini.svg"
        let icon_path = format!("/home/nia/Github/nwidgets/assets/{}", icon_filename);
        SiteData { name, url, icon_path }
    }).collect());

    let site_names: Vec<&str> = sites_with_icons.iter().map(|s| s.name).collect();
    let model = gtk::StringList::new(&site_names);

    let dropdown = gtk::DropDown::builder()
        .model(&model)
        .factory(&{
            let factory = gtk::SignalListItemFactory::new();

            factory.connect_setup(move |_, list_item| {
                let box_container = gtk::Box::new(gtk::Orientation::Horizontal, 5);
                let icon_image = gtk::Image::new();
                let label = gtk::Label::new(None);

                box_container.append(&icon_image);
                box_container.append(&label);
                list_item.set_child(Some(&box_container));
            });

            let sites_with_icons_clone = Rc::clone(&sites_with_icons);
            factory.connect_bind(move |_, list_item| {
                let box_container = list_item.child().and_then(|c| c.downcast::<gtk::Box>().ok()).unwrap();
                let icon_image = box_container.first_child().and_then(|c| c.downcast::<gtk::Image>().ok()).unwrap();
                let label = box_container.last_child().and_then(|c| c.downcast::<gtk::Label>().ok()).unwrap();

                let string_object = list_item.item().and_then(|i| i.downcast::<gtk::StringObject>().ok()).unwrap();
                let site_name = string_object.string();

                // Find the corresponding SiteData
                if let Some(index) = sites_with_icons_clone.iter().position(|s| s.name == site_name.as_str()) {
                    let site_data = &sites_with_icons_clone[index];
                    icon_image.set_from_file(Some(&site_data.icon_path));
                    label.set_text(site_data.name);
                }
            });
            factory
        })
        .build();
    dropdown.add_css_class("chat-site-dropdown");

    let webview_clone = webview.clone();
    let sites_with_icons_clone_for_dropdown = Rc::clone(&sites_with_icons);
    dropdown.connect_selected_item_notify(move |dropdown| {
        if let Some(selected) = dropdown.selected_item() {
            if let Ok(pos) = selected.downcast::<gtk::StringObject>().map(|s| s.string()) {
                if let Some(index) = sites_with_icons_clone_for_dropdown.iter().position(|s| s.name == pos.as_str()) {
                    webview_clone.load_uri(sites_with_icons_clone_for_dropdown[index].url);
                }
            }
        }
    });

    // --- Toggle Button ---
    let toggle_button = gtk::Button::with_label(ICON_RESERVE_SPACE);
    toggle_button.add_css_class("chat-pin-button");
    let is_exclusive = Rc::new(Cell::new(false));
    let is_exclusive_for_button = Rc::clone(&is_exclusive);
    let window_clone = window.clone();
    let toggle_button_clone = toggle_button.clone();
    toggle_button.connect_clicked(move |_| {
        if is_exclusive_for_button.get() {
            window_clone.set_exclusive_zone(0);
            is_exclusive_for_button.set(false);
            toggle_button_clone.set_label(ICON_RESERVE_SPACE);
        } else {
            window_clone.auto_exclusive_zone_enable();
            is_exclusive_for_button.set(true);
            toggle_button_clone.set_label(ICON_RELEASE_SPACE);
        }
    });

    // --- Layout ---
    let top_bar = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    top_bar.add_css_class("chat-top-bar");
    top_bar.append(&dropdown);
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true); // Make the spacer expand horizontally
    top_bar.append(&spacer);
    top_bar.append(&toggle_button);

    let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
    layout.add_css_class("chat-layout");
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
    let webview_clone = webview.clone();
    toggle_action.connect_activate(move |_, _| {
        let is_visible = window_clone.is_visible();
        window_clone.set_visible(!is_visible);

        // Si on ouvre la fenêtre, donner le focus au webview
        if !is_visible {
            webview_clone.grab_focus();
            println!("[CHAT] Toggle chat window: true (focus grabbed)");
        } else {
            println!("[CHAT] Toggle chat window: false");
        }
    });

    application.add_action(&toggle_action);

    // Gestionnaire de raccourci clavier Meta+P pour pin/unpin
    let key_controller = gtk::EventControllerKey::new();
    let window_clone = window.clone();
    let is_exclusive_clone = Rc::clone(&is_exclusive);
    let toggle_button_clone2 = toggle_button.clone();

    key_controller.connect_key_pressed(move |_, keyval, _, modifiers| {
        // Escape pour fermer si pas pinné
        if keyval == gtk::gdk::Key::Escape && !is_exclusive_clone.get() {
            window_clone.set_visible(false);
            println!("[CHAT] Window hidden (Escape pressed, not pinned)");
            return gtk::glib::Propagation::Stop;
        }

        // Meta+P (Super+P)
        if keyval == gtk::gdk::Key::p && modifiers.contains(gtk::gdk::ModifierType::SUPER_MASK) {
            if is_exclusive_clone.get() {
                window_clone.set_exclusive_zone(0);
                is_exclusive_clone.set(false);
                toggle_button_clone2.set_label(ICON_RESERVE_SPACE);
                println!("[CHAT] Released exclusive space (Meta+P)");
            } else {
                window_clone.auto_exclusive_zone_enable();
                is_exclusive_clone.set(true);
                toggle_button_clone2.set_label(ICON_RELEASE_SPACE);
                println!("[CHAT] Reserved exclusive space (Meta+P)");
            }
            return gtk::glib::Propagation::Stop;
        }
        gtk::glib::Propagation::Proceed
    });

    window.add_controller(key_controller);

    // Créer le PinController pour permettre le contrôle externe
    let pin_controller = PinController::new(
        window.clone(),
        Rc::clone(&is_exclusive),
        toggle_button.clone(),
        ICON_RESERVE_SPACE,
        ICON_RELEASE_SPACE,
    );

    (window, pin_controller)
}
