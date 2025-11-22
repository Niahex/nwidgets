use gtk4 as gtk;
use gtk::prelude::*;
use crate::theme::colors::COLORS;

pub fn create_dayview() -> gtk::ScrolledWindow {
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_vexpand(true);
    scrolled.set_hscrollbar_policy(gtk::PolicyType::Never);
    scrolled.set_vscrollbar_policy(gtk::PolicyType::Automatic);

    let tasks_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    tasks_box.set_margin_start(16);
    tasks_box.set_margin_end(16);
    tasks_box.set_margin_top(16);
    tasks_box.set_margin_bottom(16);

    let bg_css = gtk::CssProvider::new();
    bg_css.load_from_data(&format!(
        "box {{ background-color: {}; }}",
        COLORS.polar0.to_hex_string()
    ));
    tasks_box.style_context().add_provider(
        &bg_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let sample_task = create_sample_task("Example task", false);
    tasks_box.append(&sample_task);

    scrolled.set_child(Some(&tasks_box));
    scrolled
}

fn create_sample_task(text: &str, completed: bool) -> gtk::Box {
    let task_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    task_box.set_margin_top(8);
    task_box.set_margin_bottom(8);
    task_box.set_margin_start(12);
    task_box.set_margin_end(12);

    let task_css = gtk::CssProvider::new();
    task_css.load_from_data(&format!(
        "box {{
            background-color: {};
            border-radius: 8px;
            padding: 12px;
        }}",
        COLORS.polar2.to_hex_string()
    ));
    task_box.style_context().add_provider(
        &task_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let checkbox = gtk::CheckButton::new();
    checkbox.set_active(completed);
    task_box.append(&checkbox);

    let task_label = gtk::Label::new(Some(text));
    task_label.set_halign(gtk::Align::Start);
    task_label.set_hexpand(true);

    let label_css = gtk::CssProvider::new();
    let color = if completed {
        COLORS.polar3.to_hex_string()
    } else {
        COLORS.snow0.to_hex_string()
    };
    let decoration = if completed { "line-through" } else { "none" };

    label_css.load_from_data(&format!(
        "label {{ color: {}; text-decoration: {}; }}",
        color, decoration
    ));
    task_label.style_context().add_provider(
        &label_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    task_box.append(&task_label);

    let delete_btn = gtk::Button::new();
    delete_btn.set_label("");
    let delete_css = gtk::CssProvider::new();
    delete_css.load_from_data(&format!(
        "button {{ color: {}; background: transparent; border: none; }}",
        COLORS.red.to_hex_string()
    ));
    delete_btn.style_context().add_provider(
        &delete_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    task_box.append(&delete_btn);

    task_box
}
