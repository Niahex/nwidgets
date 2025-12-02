use once_cell::sync::OnceCell;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum OsdEvent {
    Volume(String, u8, bool), // icon_name, volume %, muted
    Microphone(bool),         // muted
    CapsLock(bool),           // enabled
    NumLock(bool),            // enabled
    Clipboard,                // copied
    #[allow(dead_code)]
    DictationStarted,
    #[allow(dead_code)]
    DictationStopped,
    SttRecording,        // STT recording started
    SttProcessing,       // STT processing
    SttComplete(String), // STT complete with transcription text
    SttError(String),    // STT error
}

static OSD_SENDER: OnceCell<Arc<Mutex<Sender<OsdEvent>>>> = OnceCell::new();

pub struct OsdEventService;

impl OsdEventService {
    /// Initialize the global OSD event channel
    pub fn init() -> Receiver<OsdEvent> {
        let (sender, receiver) = channel();
        OSD_SENDER
            .set(Arc::new(Mutex::new(sender)))
            .expect("OsdEventService::init() should only be called once");
        receiver
    }

    /// Send an OSD event (can be called from anywhere)
    pub fn send_event(event: OsdEvent) {
        if let Some(sender) = OSD_SENDER.get() {
            if let Ok(sender) = sender.lock() {
                let _ = sender.send(event);
            }
        }
    }
}

#[allow(dead_code)]
pub fn receive_osd_events(receiver: &Receiver<OsdEvent>) -> Option<OsdEvent> {
    match receiver.try_recv() {
        Ok(event) => Some(event),
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => None,
    }
}
