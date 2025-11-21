use std::{ops::Deref, rc::Rc};
use wry::{dpi::{self, LogicalSize}, Rect};
use gpui::{
    canvas, div, App, Bounds, ContentMask, DismissEvent, Element, ElementId, Entity, EventEmitter,
    FocusHandle, Focusable, GlobalElementId, Hitbox, InteractiveElement, IntoElement, LayoutId,
    MouseDownEvent, ParentElement as _, Pixels, Render, Size, Style, Styled as _, Window,
};

/// Extension trait for Pixels to provide convenient conversion methods
trait PixelsExt {
    fn as_f32(&self) -> f32;
    fn as_f64(&self) -> f64;
}

impl PixelsExt for Pixels {
    fn as_f32(&self) -> f32 {
        f32::from(*self)
    }

    fn as_f64(&self) -> f64 {
        f64::from(*self)
    }
}

pub struct WebView {
    focus_handle: FocusHandle,
    webview: Rc<wry::WebView>,
    visible: bool,
    bounds: Bounds<Pixels>,
}

impl Drop for WebView {
    fn drop(&mut self) {
        self.hide();
    }
}

impl WebView {
    pub fn new(webview: wry::WebView, _: &mut Window, cx: &mut App) -> Self {
        println!("[WEBVIEW] Initializing WebView");
        let _ = webview.set_bounds(Rect::default());
        let _ = webview.set_visible(true);
        println!("[WEBVIEW] WebView visibility set to true");

        Self {
            focus_handle: cx.focus_handle(),
            visible: true,
            bounds: Bounds::default(),
            webview: Rc::new(webview),
        }
    }

    pub fn show(&mut self) {
        let _ = self.webview.set_visible(true);
        self.visible = true;
    }

    pub fn hide(&mut self) {
        _ = self.webview.focus_parent();
        _ = self.webview.set_visible(false);
        self.visible = false;
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn bounds(&self) -> Bounds<Pixels> {
        self.bounds
    }

    pub fn load_url(&mut self, url: &str) {
        println!("[WEBVIEW] Loading URL: {}", url);
        match self.webview.load_url(url) {
            Ok(_) => println!("[WEBVIEW] URL loaded successfully"),
            Err(e) => eprintln!("[WEBVIEW] Error loading URL: {:?}", e),
        }
    }

    pub fn load_html(&mut self, html: &str) {
        println!("[WEBVIEW] Loading HTML content");
        match self.webview.load_html(html) {
            Ok(_) => println!("[WEBVIEW] HTML loaded successfully"),
            Err(e) => eprintln!("[WEBVIEW] Error loading HTML: {:?}", e),
        }
    }

    pub fn inner(&self) -> &wry::WebView {
        &self.webview
    }
}

impl Deref for WebView {
    type Target = wry::WebView;

    fn deref(&self) -> &Self::Target {
        &self.webview
    }
}

impl Focusable for WebView {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<DismissEvent> for WebView {}

impl Render for WebView {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let view = cx.entity().clone();

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .child({
                let view = cx.entity().clone();
                canvas(
                    move |bounds, _, cx| view.update(cx, |r, _| r.bounds = bounds),
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full()
            })
            .child(WebViewElement::new(self.webview.clone(), view, window, cx))
    }
}

pub struct WebViewElement {
    parent: Entity<WebView>,
    view: Rc<wry::WebView>,
}

impl WebViewElement {
    pub fn new(view: Rc<wry::WebView>, parent: Entity<WebView>, _window: &mut Window, _cx: &mut App) -> Self {
        Self { view, parent }
    }
}

impl IntoElement for WebViewElement {
    type Element = WebViewElement;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for WebViewElement {
    type RequestLayoutState = ();
    type PrepaintState = Option<Hitbox>;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.flex_grow = 0.0;
        style.flex_shrink = 1.;
        style.size = Size::full();

        let id = window.request_layout(style, [], cx);
        (id, ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        if !self.parent.read(cx).visible() {
            println!("[WEBVIEW] Parent not visible, skipping prepaint");
            return None;
        }

        let width = bounds.size.width.as_f32();
        let height = bounds.size.height.as_f32();
        let x = f32::from(bounds.origin.x);
        let y = f32::from(bounds.origin.y);

        println!("[WEBVIEW] Setting bounds: x={}, y={}, width={}, height={}", x, y, width, height);

        self.view
            .set_bounds(Rect {
                size: dpi::Size::Logical(LogicalSize {
                    width: width.into(),
                    height: height.into(),
                }),
                position: dpi::Position::Logical(dpi::LogicalPosition::new(
                    x as f64,
                    y as f64,
                )),
            })
            .unwrap();

        println!("[WEBVIEW] Bounds set successfully");

        Some(window.insert_hitbox(bounds, gpui::HitboxBehavior::Normal))
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        window: &mut Window,
        _: &mut App,
    ) {
        let bounds = hitbox.clone().map(|h| h.bounds).unwrap_or(bounds);
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            let webview = self.view.clone();
            window.on_mouse_event(move |event: &MouseDownEvent, _, _, _| {
                if !bounds.contains(&event.position) {
                    let _ = webview.focus_parent();
                }
            });
        });
    }
}
