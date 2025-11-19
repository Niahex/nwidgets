use crate::services::SpeechRecognitionService;
use crate::theme::*;
use gpui::{div, prelude::*, rgb};
use std::sync::{Arc, Mutex};

pub struct DictationModule {
    speech_service: Option<Arc<Mutex<SpeechRecognitionService>>>,
    is_recording: bool,
    last_text: String,
}

impl DictationModule {
    pub fn new() -> Self {
        Self {
            speech_service: None,
            is_recording: false,
            last_text: String::new(),
        }
    }

    /// Initialize the speech recognition service (Google Speech API)
    pub fn initialize(&mut self) -> Result<(), String> {
        println!("[DICTATION] Initializing Google Speech API service");

        match SpeechRecognitionService::new() {
            Ok(service) => {
                self.speech_service = Some(Arc::new(Mutex::new(service)));
                println!("[DICTATION] Speech service initialized successfully");
                Ok(())
            }
            Err(e) => {
                eprintln!("[DICTATION] Failed to initialize: {}", e);
                Err(format!("Failed to initialize speech service: {}", e))
            }
        }
    }

    /// Alias for initialize (for backwards compatibility)
    pub fn initialize_default(&mut self) -> Result<(), String> {
        self.initialize()
    }

    /// Toggle recording on/off
    pub fn toggle_recording(&mut self) {
        if let Some(service) = &self.speech_service {
            if self.is_recording {
                // Stop recording
                service.lock().unwrap().stop_recording();
                self.is_recording = false;
                println!("[DICTATION] Recording stopped");
            } else {
                // Start recording
                if let Err(e) = service.lock().unwrap().start_recording(move |text| {
                    // Callback when text is recognized
                    println!("[DICTATION] Recognized text: {}", text);

                    // TODO: Inject text into focused application
                    Self::inject_text(&text);
                }) {
                    eprintln!("[DICTATION] Failed to start recording: {}", e);
                    return;
                }
                self.is_recording = true;
                println!("[DICTATION] Recording started");
            }
        } else {
            eprintln!("[DICTATION] Speech service not initialized");
        }
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// Inject text into the focused application
    fn inject_text(text: &str) {
        // Use wtype to simulate keyboard typing on Wayland
        // This will type the text into whatever input has focus
        let output = std::process::Command::new("wtype")
            .arg(text)
            .output();

        match output {
            Ok(result) => {
                if !result.status.success() {
                    eprintln!("[DICTATION] wtype failed: {}",
                        String::from_utf8_lossy(&result.stderr));
                }
            }
            Err(e) => {
                eprintln!("[DICTATION] Failed to execute wtype: {}", e);
                eprintln!("[DICTATION] Make sure 'wtype' is installed: sudo pacman -S wtype");
            }
        }
    }

    /// Render the dictation indicator (shows when recording)
    pub fn render(&self) -> impl IntoElement {
        if !self.is_recording {
            // Don't show anything when not recording
            return div().hidden();
        }

        // Show recording indicator
        div()
            .w_16()
            .h_12()
            .bg(colors::red(90))
            .rounded_md()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_1()
            .child(
                div()
                    .text_xl()
                    .text_color(rgb(SNOW2))
                    .child(icons::MICROPHONE)
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(SNOW2))
                    .child("REC")
            )
    }
}
