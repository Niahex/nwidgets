use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum OsdEvent {
    Volume(u8, bool), // volume %, muted
    Microphone(bool), // muted
    DictationStarted,
    DictationStopped,
}

static mut OSD_SENDER: Option<Arc<Mutex<Sender<OsdEvent>>>> = None;

pub struct OsdEventService;

impl OsdEventService {
    /// Initialize the global OSD event channel
    pub fn init() -> Receiver<OsdEvent> {
        let (sender, receiver) = channel();
        unsafe {
            OSD_SENDER = Some(Arc::new(Mutex::new(sender)));
        }
        receiver
    }

    /// Send an OSD event (can be called from anywhere)
    pub fn send_event(event: OsdEvent) {
        unsafe {
            if let Some(sender) = &OSD_SENDER {
                if let Ok(sender) = sender.lock() {
                    let _ = sender.send(event);
                }
            }
        }
    }
}

pub fn receive_osd_events(receiver: &Receiver<OsdEvent>) -> Option<OsdEvent> {
    match receiver.try_recv() {
        Ok(event) => Some(event),
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => None,
    }
}
