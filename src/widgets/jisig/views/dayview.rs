use gtk4 as gtk;
use gtk::prelude::*;

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
    tasks_box.add_css_class("dayview-tasks");

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
    task_box.add_css_class("task-item");

    let checkbox = gtk::CheckButton::new();
    checkbox.set_active(completed);
    task_box.append(&checkbox);

    let task_label = gtk::Label::new(Some(text));
    task_label.set_halign(gtk::Align::Start);
    task_label.set_hexpand(true);
    
    if completed {
        task_label.add_css_class("task-label-completed");
    } else {
        task_label.add_css_class("task-label");
    }
    task_box.append(&task_label);

    let delete_btn = gtk::Button::new();
    delete_btn.set_label("");
    delete_btn.add_css_class("task-delete-button");
    task_box.append(&delete_btn);

    task_box
}
