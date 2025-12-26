use super::audio_details::PanelManager;
use gtk::prelude::*;
use gtk4 as gtk;

/// Setup expand/collapse callback for a control center section
///
/// This helper reduces boilerplate for sections that have expand buttons.
pub fn setup_expand_callback<F>(
    expanded: &gtk::Box,
    expand_btn: &gtk::Button,
    panels: &PanelManager,
    panel_name: &str,
    populate_fn: F,
) where
    F: Fn(&gtk::Box) + 'static,
{
    let panels_clone = panels.clone();
    let expanded_clone = expanded.clone();
    let panel_name = panel_name.to_string();

    expand_btn.connect_clicked(move |btn| {
        let is_visible = expanded_clone.is_visible();
        if !is_visible {
            panels_clone.collapse_all_except(&panel_name);
            expanded_clone.set_visible(true);
            btn.set_icon_name("go-up-symbolic");
            populate_fn(&expanded_clone);
        } else {
            expanded_clone.set_visible(false);
            btn.set_icon_name("go-down-symbolic");
        }
    });
}

/// Setup periodic updates for a section when visible
///
/// This helper creates a timer that calls the populate function every
/// `interval` seconds, but only when the section is visible.
pub fn setup_periodic_updates<F>(expanded: &gtk::Box, interval_secs: u64, populate_fn: F)
where
    F: Fn(&gtk::Box) + 'static,
{
    let expanded_clone = expanded.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_secs(interval_secs), move || {
        if expanded_clone.is_visible() {
            populate_fn(&expanded_clone);
        }
        gtk::glib::ControlFlow::Continue
    });
}
