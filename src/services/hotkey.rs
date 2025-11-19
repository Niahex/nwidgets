use evdev::{Device, InputEventKind, Key};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    DictationToggle,
}

pub struct HotkeyService {
    receiver: Receiver<HotkeyEvent>,
}

impl HotkeyService {
    /// Start listening for hotkey events
    /// Returns a service that can be polled for events
    pub fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let (sender, receiver) = channel();

        // Spawn a thread to listen for keyboard events
        thread::spawn(move || {
            if let Err(e) = Self::listen_for_keys(sender) {
                eprintln!("[HOTKEY] Error: {}", e);
            }
        });

        Ok(Self { receiver })
    }

    /// Check if a hotkey event has been triggered
    pub fn poll_event(&self) -> Option<HotkeyEvent> {
        self.receiver.try_recv().ok()
    }

    fn listen_for_keys(sender: Sender<HotkeyEvent>) -> Result<(), Box<dyn std::error::Error>> {
        // Find keyboard device
        let mut keyboard = Self::find_keyboard_device()?;

        println!("[HOTKEY] Listening for Super+Space to toggle dictation");

        let mut super_pressed = false;

        loop {
            for event in keyboard.fetch_events()? {
                if let InputEventKind::Key(key) = event.kind() {
                    let pressed = event.value() == 1; // 1 = pressed, 0 = released

                    match key {
                        // Track Super key state (left or right)
                        Key::KEY_LEFTMETA | Key::KEY_RIGHTMETA => {
                            super_pressed = pressed;
                        }
                        // Space key
                        Key::KEY_SPACE => {
                            if pressed && super_pressed {
                                println!("[HOTKEY] Super+Space detected - toggling dictation");
                                sender.send(HotkeyEvent::DictationToggle).ok();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn find_keyboard_device() -> Result<Device, Box<dyn std::error::Error>> {
        // Try to find a keyboard device
        for entry in std::fs::read_dir("/dev/input")? {
            let entry = entry?;
            let path = entry.path();

            if let Ok(mut device) = Device::open(&path) {
                // Check if this device has keyboard capabilities
                if device.supported_keys().map_or(false, |keys| {
                    keys.contains(Key::KEY_SPACE) && keys.contains(Key::KEY_LEFTMETA)
                }) {
                    println!("[HOTKEY] Using keyboard device: {:?}", path);

                    // Grab the device to prevent other applications from receiving these events
                    // This prevents the hotkey from being sent to other apps
                    if let Err(e) = device.grab() {
                        eprintln!("[HOTKEY] Warning: Could not grab device: {}. You may need to run with elevated permissions or add your user to the 'input' group.", e);
                        eprintln!("[HOTKEY] Run: sudo usermod -a -G input $USER && newgrp input");
                    }

                    return Ok(device);
                }
            }
        }

        Err("No suitable keyboard device found".into())
    }
}
