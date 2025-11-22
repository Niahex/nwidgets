use gtk4 as gtk;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::collections::HashSet;
use std::env;
use crate::services::ApplicationsService;
use crate::theme::colors::COLORS;

pub fn create_launcher_window(application: &gtk::Application) -> gtk::ApplicationWindow {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("Nwidgets Launcher")
        .default_width(500)
        .default_height(400)
        .build();

    // --- GTK Layer Shell Setup ---
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Top, false);
    window.set_anchor(Edge::Bottom, false);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);
    window.set_margin(Edge::Top, 100);

    // Force keyboard focus
    if env::var("GTK_DEBUG").unwrap_or_default() != "interactive" {
        window.set_keyboard_mode(KeyboardMode::Exclusive);
    }

    // --- Container ---
    let container = gtk::Box::new(gtk::Orientation::Vertical, 10);
    container.set_margin_start(20);
    container.set_margin_end(20);
    container.set_margin_top(20);
    container.set_margin_bottom(20);

    let container_css = gtk::CssProvider::new();
    container_css.load_from_data(&format!(
        "box {{
            background-color: {};
            border: 1px solid {};
            border-radius: 10px;
            padding: 10px;
        }}",
        COLORS.polar0.to_hex_string(),
        COLORS.frost1.with_opacity(25)
    ));
    container.style_context().add_provider(
        &container_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // --- Search Entry ---
    let search_entry = gtk::Entry::builder()
        .placeholder_text("Search applications...")
        .can_focus(true)
        .build();

    search_entry.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, Some("system-search-symbolic"));

    let search_css = gtk::CssProvider::new();
    search_css.load_from_data(&format!(
        "entry {{
            background-color: {};
            border-radius: 5px;
            padding: 5px;
            color: {};
        }}",
        COLORS.polar2.to_hex_string(),
        COLORS.snow0.to_hex_string()
    ));
    search_entry.style_context().add_provider(
        &search_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // --- ListView Setup ---
    // Load from cache first for instant display
    let app_list_store = ApplicationsService::get_cached_applications();

    // Create filter
    let app_filter = gtk::CustomFilter::new({
        let search_entry = search_entry.clone();
        move |obj| {
            let app_info = obj.downcast_ref::<gtk::gio::AppInfo>().unwrap();
            let search_text = search_entry.text().to_lowercase();
            if search_text.is_empty() {
                return true;
            }
            app_info.name().to_lowercase().contains(&search_text)
        }
    });

    let filtered_model = gtk::FilterListModel::new(Some(app_list_store.clone()), Some(app_filter.clone()));
    let selection_model = gtk::SingleSelection::new(Some(filtered_model.clone()));

    // Create factory for list items
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 10);

        let icon = gtk::Image::new();
        icon.set_icon_size(gtk::IconSize::Large);

        let label = gtk::Label::new(None);
        label.set_halign(gtk::Align::Start);

        hbox.append(&icon);
        hbox.append(&label);
        item.set_child(Some(&hbox));
    });

    factory.connect_bind(move |_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let hbox = item.child().and_downcast::<gtk::Box>().unwrap();
        let icon = hbox.first_child().and_downcast::<gtk::Image>().unwrap();
        let label = hbox.last_child().and_downcast::<gtk::Label>().unwrap();
        let app_info = item.item().and_downcast::<gtk::gio::AppInfo>().unwrap();

        if let Some(gicon) = app_info.icon() {
            icon.set_from_gicon(&gicon);
        } else {
            icon.set_icon_name(Some("application-x-executable"));
        }
        label.set_text(&app_info.name());

        let label_css = gtk::CssProvider::new();
        label_css.load_from_data(&format!(
            "label {{ color: {}; }}",
            COLORS.snow0.to_hex_string()
        ));
        label.style_context().add_provider(
            &label_css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    let list_view = gtk::ListView::new(Some(selection_model.clone()), Some(factory));

    let list_css = gtk::CssProvider::new();
    list_css.load_from_data(&format!(
        "listview row:selected {{
            background-color: {};
            color: {};
            border-radius: 5px;
        }}",
        COLORS.frost1.to_hex_string(),
        COLORS.polar0.to_hex_string()
    ));
    list_view.style_context().add_provider(
        &list_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Scrolled window for list
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .min_content_height(300)
        .child(&list_view)
        .vexpand(true)
        .build();

    container.append(&search_entry);
    container.append(&scrolled);

    window.set_child(Some(&container));

    // --- Event Handlers ---

    // Update filter when search text changes
    let app_filter_clone = app_filter.clone();
    let selection_model_clone = selection_model.clone();
    search_entry.connect_changed(move |_| {
        app_filter_clone.changed(gtk::FilterChange::Different);
        gtk::glib::idle_add_local_once({
            let selection_model = selection_model_clone.clone();
            move || {
                if selection_model.n_items() > 0 {
                    selection_model.set_selected(0);
                }
            }
        });
    });

    // Launch app on Enter
    let application_clone = application.clone();
    let selection_model_clone = selection_model.clone();
    search_entry.connect_activate(move |_| {
        launch_selected_app(&application_clone, &selection_model_clone);
    });

    // Keyboard navigation
    let key_controller = gtk::EventControllerKey::new();
    let application_clone = application.clone();
    let list_view_clone = list_view.clone();
    let selection_model_clone = selection_model.clone();
    key_controller.connect_key_pressed(move |_, keyval, _, _| {
        match keyval {
            gtk::gdk::Key::Escape => {
                application_clone.lookup_action("toggle-launcher")
                    .and_downcast::<gtk::gio::SimpleAction>()
                    .unwrap()
                    .activate(None);
                gtk::glib::Propagation::Stop
            },
            gtk::gdk::Key::Down => {
                navigate_list(&selection_model_clone, &list_view_clone, 1);
                gtk::glib::Propagation::Stop
            },
            gtk::gdk::Key::Up => {
                navigate_list(&selection_model_clone, &list_view_clone, -1);
                gtk::glib::Propagation::Stop
            },
            _ => gtk::glib::Propagation::Proceed,
        }
    });
    window.add_controller(key_controller);

    // Clear search on hide
    let search_entry_clone = search_entry.clone();
    window.connect_hide(move |_| {
        search_entry_clone.set_text("");
    });

    // Focus search entry when shown
    let search_entry_clone = search_entry.clone();
    let selection_model_clone = selection_model.clone();
    window.connect_show(move |_| {
        if env::var("GTK_DEBUG").unwrap_or_default() != "interactive" {
            search_entry_clone.grab_focus();
        }
        if selection_model_clone.n_items() > 0 {
            selection_model_clone.set_selected(0);
        }
    });

    // Hide by default
    window.set_visible(false);

    // Add toggle action
    let toggle_action = gtk::gio::SimpleAction::new("toggle-launcher", None);
    let window_clone = window.clone();
    toggle_action.connect_activate(move |_, _| {
        let is_visible = window_clone.is_visible();
        window_clone.set_visible(!is_visible);
        if !is_visible {
            println!("[LAUNCHER] Toggle launcher window: true");
        } else {
            println!("[LAUNCHER] Toggle launcher window: false");
        }
    });
    application.add_action(&toggle_action);

    // --- Subscribe to application updates ---
    let app_list_store_clone = app_list_store.clone();
    ApplicationsService::start_monitoring(move |app_ids| {
        let cached_app_ids: HashSet<String> = app_list_store_clone
            .iter::<gtk::gio::AppInfo>()
            .filter_map(|app_info| app_info.ok())
            .filter_map(|app_info| app_info.id().map(|s| s.to_string()))
            .collect();

        let new_app_ids: HashSet<String> = app_ids.iter().cloned().collect();

        // Add new apps
        for id in new_app_ids.difference(&cached_app_ids) {
            if let Some(desktop_app_info) = gtk::gio::DesktopAppInfo::new(id) {
                let app_info = desktop_app_info.upcast::<gtk::gio::AppInfo>();
                app_list_store_clone.append(&app_info);
            }
        }

        // Remove old apps
        let mut to_remove = Vec::new();
        for (i, app_info) in app_list_store_clone.iter::<gtk::gio::AppInfo>().enumerate() {
            if let Ok(app_info) = app_info {
                if let Some(id) = app_info.id() {
                    if !new_app_ids.contains(id.as_str()) {
                        to_remove.push(i as u32);
                    }
                }
            }
        }
        to_remove.reverse(); // Remove from the end to avoid index shifting
        for i in to_remove {
            app_list_store_clone.remove(i);
        }

        println!("[LAUNCHER] Applications updated. Total: {}", app_ids.len());
    });

    window
}

fn navigate_list(selection_model: &gtk::SingleSelection, _list_view: &gtk::ListView, direction: i32) {
    let current_pos = selection_model.selected();
    let n_items = selection_model.n_items();
    if n_items == 0 {
        return;
    }

    let next_pos = if current_pos == gtk::INVALID_LIST_POSITION {
        if direction > 0 { 0 } else { n_items - 1 }
    } else {
        ((current_pos as i32 + direction + n_items as i32) as u32) % n_items
    };

    selection_model.set_selected(next_pos);
}

fn launch_selected_app(application: &gtk::Application, selection_model: &gtk::SingleSelection) {
    if let Some(selected_item) = selection_model.selected_item() {
        if let Some(app_info) = selected_item.downcast_ref::<gtk::gio::AppInfo>() {
            match app_info.launch(&[], gtk::gio::AppLaunchContext::NONE) {
                Ok(_) => {
                    println!("[LAUNCHER] Launched: {}", app_info.name());
                    // Close launcher after launching app
                    application.lookup_action("toggle-launcher")
                        .and_downcast::<gtk::gio::SimpleAction>()
                        .unwrap()
                        .activate(None);
                },
                Err(e) => {
                    eprintln!("[LAUNCHER] Error launching application: {}", e);
                }
            }
        }
    }
}
