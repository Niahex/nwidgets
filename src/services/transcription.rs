use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum TranscriptionEvent {
    TextRecognized(String),
    StopRequested, // Request to stop recording from UI
}

static mut TRANSCRIPTION_SENDER: Option<Arc<Mutex<Sender<TranscriptionEvent>>>> = None;

pub struct TranscriptionEventService;

impl TranscriptionEventService {
    /// Initialize the global transcription event channel
    pub fn init() -> Receiver<TranscriptionEvent> {
        let (sender, receiver) = channel();
        unsafe {
            TRANSCRIPTION_SENDER = Some(Arc::new(Mutex::new(sender)));
        }
        receiver
    }

    /// Send a transcription event (can be called from anywhere)
    pub fn send_event(event: TranscriptionEvent) {
        unsafe {
            if let Some(sender) = &TRANSCRIPTION_SENDER {
                if let Ok(sender) = sender.lock() {
                    let _ = sender.send(event);
                }
            }
        }
    }
}

pub fn receive_transcription_events(receiver: &Receiver<TranscriptionEvent>) -> Option<TranscriptionEvent> {
    match receiver.try_recv() {
        Ok(event) => Some(event),
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => None,
    }
}
