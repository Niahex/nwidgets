use cef::{wrap_client, Client, ImplClient, WrapClient, LifeSpanHandler, RenderHandler};
use cef::rc::Rc;
use super::browser::CefBrowser;

wrap_client! {
    pub struct CefClient;

    impl Client {
        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(CefLifeSpanHandler::new())
        }

        fn render_handler(&self) -> Option<RenderHandler> {
            Some(CefRenderHandler::new())
        }
    }
}

use cef::{wrap_life_span_handler, ImplLifeSpanHandler, WrapLifeSpanHandler, Browser};

wrap_life_span_handler! {
    struct CefLifeSpanHandler;

    impl LifeSpanHandler {
        fn on_after_created(&self, browser: Option<&mut Browser>) {
            if let Some(browser) = browser {
                CefBrowser::set(browser.clone());
            }
        }
    }
}

use cef::{wrap_render_handler, ImplRenderHandler, WrapRenderHandler, PaintElementType, Rect};

wrap_render_handler! {
    struct CefRenderHandler;

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                rect.x = 0;
                rect.y = 0;
                rect.width = 600;
                rect.height = 1440;
            }
        }

        fn on_paint(
            &self,
            _browser: Option<&mut Browser>,
            _type_: PaintElementType,
            _dirty_rects: Option<&[Rect]>,
            _buffer: *const u8,
            _width: i32,
            _height: i32,
        ) {
            // TODO: Copy buffer to GPUI texture
        }
    }
}
