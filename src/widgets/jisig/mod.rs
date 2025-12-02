use crate::utils::icons;
use crate::utils::PinController;
use gtk4 as gtk;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::Cell;
use std::env;
use std::fs;
use std::rc::Rc;
use webkit6::prelude::*;
use webkit6::{
    CookiePersistentStorage, HardwareAccelerationPolicy, Settings, UserMediaPermissionRequest,
    WebContext, WebView,
};

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
    webview.add_css_class("jisig-webview");

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
        .title("Nwidgets jisig") // Changed title
        .default_width(500)
        .default_height(600)
        .build();
    window.add_css_class("jisig-window");

    // --- GTK Layer Shell Setup ---
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Right, true);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);

    // --- Setup WebBridge ---
    crate::utils::webbridge::setup_webbridge(&webview, &window);

    // --- Toggle Button ---
    let pin_icon = icons::create_icon("pin");
    let toggle_button = gtk::Button::new();
    toggle_button.set_child(Some(&pin_icon));
    toggle_button.add_css_class("jisig-pin-button");
    let is_exclusive = Rc::new(Cell::new(false));
    let is_exclusive_for_button = Rc::clone(&is_exclusive);
    let window_clone = window.clone();
    let pin_icon_clone = pin_icon.clone();
    toggle_button.connect_clicked(move |_| {
        if is_exclusive_for_button.get() {
            window_clone.set_exclusive_zone(0);
            is_exclusive_for_button.set(false);
            if let Some(paintable) = icons::get_paintable("pin") {
                pin_icon_clone.set_paintable(Some(&paintable));
            }
        } else {
            window_clone.auto_exclusive_zone_enable();
            is_exclusive_for_button.set(true);
            if let Some(paintable) = icons::get_paintable("unpin") {
                pin_icon_clone.set_paintable(Some(&paintable));
            }
        }
    });

    // --- Layout ---
    let top_bar = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    top_bar.add_css_class("jisig-top-bar");
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true); // Make the spacer expand horizontally
    top_bar.append(&spacer);
    top_bar.append(&toggle_button);

    let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
    layout.add_css_class("jisig-layout");
    layout.append(&top_bar);
    layout.append(&webview);
    webview.set_vexpand(true);

    // Set the layout as the child of the window
    window.set_child(Some(&layout));

    // Load the initial URL
    webview.load_uri(DEFAULT_URL);

    // Cacher la fenêtre par défaut au démarrage
    window.set_visible(false);

    // Ajouter l'action toggle-jisig
    let toggle_action = gtk::gio::SimpleAction::new("toggle-jisig", None);
    let window_clone = window.clone();
    let webview_clone = webview.clone();
    toggle_action.connect_activate(move |_, _| {
        let is_visible = window_clone.is_visible();
        window_clone.set_visible(!is_visible);

        // Si on ouvre la fenêtre, donner le focus et grab_focus
        if !is_visible {
            window_clone.present();
            window_clone.grab_focus();
            webview_clone.grab_focus();
            println!("[jisig] Toggle jisig window: true (focus grabbed)");
        } else {
            println!("[jisig] Toggle jisig window: false");
        }
    });

    application.add_action(&toggle_action);

    // Gestionnaire de raccourci clavier Meta+P pour pin/unpin
    let key_controller = gtk::EventControllerKey::new();
    let window_clone = window.clone();
    let is_exclusive_clone = Rc::clone(&is_exclusive);

    let pin_icon_clone2 = pin_icon.clone();
    key_controller.connect_key_pressed(move |_, keyval, _, modifiers| {
        // Escape pour fermer si pas pinné
        if keyval == gtk::gdk::Key::Escape && !is_exclusive_clone.get() {
            window_clone.set_visible(false);
            println!("[jisig] Window hidden (Escape pressed, not pinned)");
            return gtk::glib::Propagation::Stop;
        }

        // Meta+P (Super+P)
        if keyval == gtk::gdk::Key::p && modifiers.contains(gtk::gdk::ModifierType::SUPER_MASK) {
            if is_exclusive_clone.get() {
                window_clone.set_exclusive_zone(0);
                is_exclusive_clone.set(false);
                if let Some(paintable) = icons::get_paintable("pin") {
                    pin_icon_clone2.set_paintable(Some(&paintable));
                }
                println!("[jisig] Released exclusive space (Meta+P)");
            } else {
                window_clone.auto_exclusive_zone_enable();
                is_exclusive_clone.set(true);
                if let Some(paintable) = icons::get_paintable("unpin") {
                    pin_icon_clone2.set_paintable(Some(&paintable));
                }
                println!("[jisig] Reserved exclusive space (Meta+P)");
            }
            return gtk::glib::Propagation::Stop;
        }
        gtk::glib::Propagation::Proceed
    });

    window.add_controller(key_controller);

    // Créer le PinController pour permettre le contrôle externe
    let pin_controller =
        PinController::new(window.clone(), Rc::clone(&is_exclusive), pin_icon.clone());

    JisigOverlay {
        window,
        pin_controller,
        id: "jisig-overlay-main".to_string(),
        is_pinned: is_exclusive,
    }
}
