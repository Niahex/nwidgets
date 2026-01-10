use cef::{
    rc::Rc, Browser, CefString, DisplayHandler, Frame, ImplDisplayHandler, ImplMediaAccessCallback,
    ImplPermissionHandler, ImplRenderHandler, MediaAccessCallback, PermissionHandler,
    RenderHandler, WrapDisplayHandler, WrapPermissionHandler, WrapRenderHandler,
};
use cef_dll_sys::cef_cursor_type_t;
use parking_lot::Mutex;
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

#[derive(Clone)]
pub struct GpuiRenderHandler {
    pub pixels: Arc<Mutex<Vec<u8>>>,
    pub width: Arc<Mutex<u32>>,
    pub height: Arc<Mutex<u32>>,
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

        fn on_paint(
            &self,
            _browser: Option<&mut Browser>,
            _type_: cef::PaintElementType,
            _dirty_rects: Option<&[cef::Rect]>,
            buffer: *const u8,
            width: i32,
            height: i32,
        ) {
            if buffer.is_null() { return; }
            let len = (width * height * 4) as usize;
            let src = unsafe { std::slice::from_raw_parts(buffer, len) };
            let mut pixels = self.handler.pixels.lock();
            if pixels.len() != len { pixels.resize(len, 0); }
            pixels.copy_from_slice(src);
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
