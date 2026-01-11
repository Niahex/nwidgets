use cef::{
    rc::Rc, Browser, CefString, DisplayHandler, Frame, ImplDisplayHandler, ImplFrame,
    ImplLoadHandler, ImplMediaAccessCallback, ImplPermissionHandler, ImplRenderHandler,
    LoadHandler, MediaAccessCallback, PermissionHandler, RenderHandler, ScreenInfo,
    WrapDisplayHandler, WrapLoadHandler, WrapPermissionHandler, WrapRenderHandler,
};
use cef_dll_sys::cef_cursor_type_t;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CefCursor {
    Default,
    Pointer,
    Text,
    Move,
    Wait,
    None,
}

/// Double buffer for lock-free rendering
pub struct DoubleBuffer {
    buffers: [Mutex<Vec<u8>>; 2],
    active: AtomicUsize,
    version: AtomicU64,
}

impl DoubleBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffers: [
                Mutex::new(Vec::with_capacity(capacity)),
                Mutex::new(Vec::with_capacity(capacity)),
            ],
            active: AtomicUsize::new(0),
            version: AtomicU64::new(0),
        }
    }

    /// Write to back buffer and swap
    pub fn write(&self, data: &[u8]) {
        let back = 1 - self.active.load(Ordering::Acquire);
        {
            let mut buf = self.buffers[back].lock();
            if buf.len() != data.len() {
                buf.resize(data.len(), 0);
            }
            buf.copy_from_slice(data);
        }
        self.active.store(back, Ordering::Release);
        self.version.fetch_add(1, Ordering::Release);
    }

    /// Read from front buffer (non-blocking if writer is on back buffer)
    pub fn read(&self) -> parking_lot::MutexGuard<'_, Vec<u8>> {
        let front = self.active.load(Ordering::Acquire);
        self.buffers[front].lock()
    }

    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }
}

#[derive(Clone)]
pub struct GpuiRenderHandler {
    pub buffer: Arc<DoubleBuffer>,
    pub width: Arc<Mutex<u32>>,
    pub height: Arc<Mutex<u32>>,
    pub scale_factor: f32,
    pub selected_text: Arc<Mutex<String>>,
    pub repaint_tx: futures::channel::mpsc::UnboundedSender<()>,
}

cef::wrap_render_handler! {
    pub struct RenderHandlerWrapper {
        handler: GpuiRenderHandler,
    }

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut cef::Rect>) {
            if let Some(rect) = rect {
                rect.width = *self.handler.width.lock() as i32;
                rect.height = *self.handler.height.lock() as i32;
            }
        }

        fn screen_info(
            &self,
            _browser: Option<&mut Browser>,
            screen_info: Option<&mut ScreenInfo>,
        ) -> i32 {
            if let Some(info) = screen_info {
                info.device_scale_factor = self.handler.scale_factor;
                return 1;
            }
            0
        }

        fn on_paint(
            &self,
            _browser: Option<&mut Browser>,
            _type_: cef::PaintElementType,
            dirty_rects: Option<&[cef::Rect]>,
            buffer: *const u8,
            width: i32,
            height: i32,
        ) {
            if buffer.is_null() || width <= 0 || height <= 0 { return; }

            let total_len = (width * height * 4) as usize;
            let src = unsafe { std::slice::from_raw_parts(buffer, total_len) };

            // Check if we can do partial update
            if let Some(rects) = dirty_rects {
                if rects.len() == 1 && rects[0].width == width && rects[0].height == height {
                    // Full repaint - just copy everything
                    self.handler.buffer.write(src);
                } else if !rects.is_empty() {
                    // Partial update - copy dirty regions only
                    let front = self.handler.buffer.read();
                    if front.len() == total_len {
                        drop(front); // Release read lock

                        // Get back buffer for writing
                        let back_idx = 1 - self.handler.buffer.active.load(Ordering::Acquire);
                        let mut back = self.handler.buffer.buffers[back_idx].lock();

                        if back.len() != total_len {
                            back.resize(total_len, 0);
                        }

                        // Copy from front to back first (preserve unchanged areas)
                        let front = self.handler.buffer.buffers[self.handler.buffer.active.load(Ordering::Acquire)].lock();
                        back.copy_from_slice(&front);
                        drop(front);

                        // Apply dirty rects
                        let stride = (width * 4) as usize;
                        for rect in rects {
                            let rx = rect.x.max(0) as usize;
                            let ry = rect.y.max(0) as usize;
                            let rw = rect.width.min(width - rect.x) as usize;
                            let rh = rect.height.min(height - rect.y) as usize;

                            for row in 0..rh {
                                let src_offset = (ry + row) * stride + rx * 4;
                                let dst_offset = src_offset;
                                let len = rw * 4;
                                if src_offset + len <= total_len && dst_offset + len <= back.len() {
                                    back[dst_offset..dst_offset + len]
                                        .copy_from_slice(&src[src_offset..src_offset + len]);
                                }
                            }
                        }
                        drop(back);

                        // Swap buffers
                        self.handler.buffer.active.store(back_idx, Ordering::Release);
                        self.handler.buffer.version.fetch_add(1, Ordering::Release);
                    } else {
                        // Buffer size mismatch, do full copy
                        drop(front);
                        self.handler.buffer.write(src);
                    }
                }
            } else {
                // No dirty rects info, full copy
                self.handler.buffer.write(src);
            }

            // Notify UI thread
            let _ = self.handler.repaint_tx.unbounded_send(());
        }

        fn on_text_selection_changed(
            &self,
            _browser: Option<&mut Browser>,
            selected_text: Option<&CefString>,
            _selected_range: Option<&cef::Range>,
        ) {
            if let Some(text) = selected_text {
                *self.handler.selected_text.lock() = text.to_string();
            } else {
                self.handler.selected_text.lock().clear();
            }
        }
    }
}

#[derive(Clone)]
pub struct GpuiDisplayHandler {
    pub cursor: Arc<Mutex<CefCursor>>,
}

cef::wrap_display_handler! {
    pub struct DisplayHandlerWrapper {
        handler: GpuiDisplayHandler,
    }

    impl DisplayHandler {
        fn on_cursor_change(
            &self,
            _browser: Option<&mut Browser>,
            _cursor: cef::CursorHandle,
            type_: cef::CursorType,
            _custom_cursor_info: Option<&cef::CursorInfo>,
        ) -> i32 {
            let cursor = match type_.as_ref() {
                cef_cursor_type_t::CT_POINTER => CefCursor::Default,
                cef_cursor_type_t::CT_HAND => CefCursor::Pointer,
                cef_cursor_type_t::CT_IBEAM => CefCursor::Text,
                cef_cursor_type_t::CT_MOVE | cef_cursor_type_t::CT_MIDDLEPANNING => CefCursor::Move,
                cef_cursor_type_t::CT_WAIT | cef_cursor_type_t::CT_PROGRESS => CefCursor::Wait,
                cef_cursor_type_t::CT_NONE => CefCursor::None,
                _ => CefCursor::Default,
            };
            *self.handler.cursor.lock() = cursor;
            0
        }
    }
}

#[derive(Clone)]
pub struct GpuiPermissionHandler;

cef::wrap_permission_handler! {
    pub struct PermissionHandlerWrapper {
        handler: GpuiPermissionHandler,
    }

    impl PermissionHandler {
        fn on_request_media_access_permission(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _requesting_url: Option<&CefString>,
            requested_permissions: u32,
            callback: Option<&mut MediaAccessCallback>,
        ) -> i32 {
            if let Some(callback) = callback {
                callback.cont(requested_permissions);
            }
            1
        }
    }
}

#[derive(Clone)]
pub struct GpuiLoadHandler {
    pub injection_script: Arc<Mutex<Option<String>>>,
    pub loaded: Arc<Mutex<bool>>,
}

cef::wrap_load_handler! {
    pub struct LoadHandlerWrapper {
        handler: GpuiLoadHandler,
    }

    impl LoadHandler {
        fn on_load_end(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            _http_status_code: i32,
        ) {
            if let Some(frame) = frame {
                if frame.is_main() != 0 {
                    // Directly execute the pre-calculated script
                    if let Some(script) = self.handler.injection_script.lock().as_ref() {
                        frame.execute_java_script(Some(&CefString::from(script.as_str())), None, 0);
                    }
                    *self.handler.loaded.lock() = true;
                }
            }
        }
    }
}
