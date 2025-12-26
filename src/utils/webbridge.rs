use gtk4 as gtk;
use webkit6::prelude::*;
use webkit6::{UserContentInjectedFrames, UserScript, UserScriptInjectionTime};

/// Configure le WebBridge pour permettre la communication entre le contenu web et GTK
pub fn setup_webbridge(webview: &webkit6::WebView, window: &gtk::ApplicationWindow) {
    let user_content_manager = webview.user_content_manager()
        .expect("Failed to get user content manager");

    // Inject JavaScript pour exposer l'API de redimensionnement
    let resize_script = r#"
        window.webkit = window.webkit || {};
        window.webkit.messageHandlers = window.webkit.messageHandlers || {};
        window.webkit.messageHandlers.resizeWindow = {
            postMessage: function(message) {
                console.log('Resize request:', message);
                // Le message sera intercepté par le signal script-message-received
                window.webkit.messageHandlers.resizeWindow._postMessage(JSON.stringify(message));
            }
        };
    "#;

    let script = UserScript::new(
        resize_script,
        UserContentInjectedFrames::AllFrames,
        UserScriptInjectionTime::Start,
        &[],
        &[],
    );

    user_content_manager.add_script(&script);

    // Register script message handler
    user_content_manager.register_script_message_handler("resizeWindow", None);

    // Handle resize messages
    let window_clone = window.clone();
    user_content_manager.connect_script_message_received(
        Some("resizeWindow"),
        move |_, js_value| {
            if let Some(json_str) = js_value.to_json(0) {
                println!("[WebBridge] Received resize message: {}", json_str);

                // Parse le JSON pour extraire width et height
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str.as_str()) {
                    if let (Some(width), Some(height)) = (
                        parsed["width"].as_f64(),
                        parsed["height"].as_f64(),
                    ) {
                        let width = width as i32;
                        let height = height as i32;

                        println!("[WebBridge] Resizing window to {}x{}", width, height);
                        window_clone.set_default_size(width, height);
                    }
                }
            }
        },
    );

    println!("[WebBridge] WebBridge configured successfully");
}
