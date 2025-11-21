mod widgets;

use crate::widgets::chat::create_chat_window;
use crate::widgets::panel::create_panel_window;
use gtk4::{prelude::*, Application};

const APP_ID: &str = "com.nwidgets";

fn main() {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(|app| {
        let chat_window = create_chat_window(app);
        chat_window.present();

        let panel_window = create_panel_window(app);
        panel_window.present();
    });

    // Run the application
    app.run();
}
