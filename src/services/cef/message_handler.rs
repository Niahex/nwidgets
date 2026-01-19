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
        eprintln!("[CEF Clipboard] Received query {}: {}", query_id, &request[..request.len().min(100)]);
        
        // Try to parse clipboard data from the request
        if let Some(data) = super::clipboard::extract_clipboard_from_message(request) {
            eprintln!("[CEF Clipboard] Parsed clipboard data: {:?}", match &data {
                ClipboardData::Text(t) => format!("Text({}...)", &t[..t.len().min(50)]),
                ClipboardData::Image { format, data } => format!("Image({:?}, {} bytes)", format, data.len()),
            });
            
            // Send to clipboard channel
            if let Err(e) = self.clipboard_tx.unbounded_send(data) {
                eprintln!("[CEF Clipboard] Failed to send to channel: {}", e);
                if let Ok(cb) = callback.lock() {
                    cb.failure(-1, "Failed to process clipboard data");
                }
                return true;
            }
            
            // Respond success to JavaScript
            if let Ok(cb) = callback.lock() {
                cb.success_str("ok");
            }
            return true;
        }
        
        eprintln!("[CEF Clipboard] Failed to parse clipboard data from request");
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
        eprintln!("[CEF Clipboard] Received binary query {}: {} bytes", query_id, request.data().len());
        
        // Try to parse as JSON string first
        if let Ok(request_str) = std::str::from_utf8(request.data()) {
            return self.on_query_str(_browser, _frame, query_id, request_str, _persistent, callback);
        }
        
        eprintln!("[CEF Clipboard] Binary data is not valid UTF-8");
        false
    }

    fn on_query_canceled(&self, _browser: Option<Browser>, _frame: Option<Frame>, query_id: i64) {
        eprintln!("[CEF Clipboard] Query {} canceled", query_id);
    }
}
