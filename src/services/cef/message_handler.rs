use cef::wrapper::message_router::{BrowserSideCallback, BrowserSideHandler, BinaryBuffer};
use cef::{Browser, Frame};
use futures::channel::mpsc::UnboundedSender;
use std::sync::{Arc, Mutex};

use super::clipboard::ClipboardData;

/// Handler for clipboard operations via CEF MessageRouter
/// Handles both text and binary (image) clipboard data
pub struct ClipboardMessageHandler {
    clipboard_tx: UnboundedSender<ClipboardData>,
}

impl ClipboardMessageHandler {
    pub fn new(clipboard_tx: UnboundedSender<ClipboardData>) -> Self {
        Self { clipboard_tx }
    }
}

impl BrowserSideHandler for ClipboardMessageHandler {
    fn on_query_str(
        &self,
        _browser: Option<Browser>,
        _frame: Option<Frame>,
        query_id: i64,
        request: &str,
        _persistent: bool,
        callback: Arc<Mutex<dyn BrowserSideCallback>>,
    ) -> bool {
        if let Some(data) = super::clipboard::extract_clipboard_from_message(request) {
            if let Err(_e) = self.clipboard_tx.unbounded_send(data) {
                if let Ok(cb) = callback.lock() {
                    cb.failure(-1, "Failed to process clipboard data");
                }
                return true;
            }
            
            if let Ok(cb) = callback.lock() {
                cb.success_str("ok");
            }
            return true;
        }
        
        false
    }

    fn on_query_binary(
        &self,
        _browser: Option<Browser>,
        _frame: Option<Frame>,
        query_id: i64,
        request: &dyn BinaryBuffer,
        _persistent: bool,
        callback: Arc<Mutex<dyn BrowserSideCallback>>,
    ) -> bool {
        if let Ok(request_str) = std::str::from_utf8(request.data()) {
            return self.on_query_str(_browser, _frame, query_id, request_str, _persistent, callback);
        }
        false
    }

    fn on_query_canceled(&self, _browser: Option<Browser>, _frame: Option<Frame>, _query_id: i64) {
    }
}
