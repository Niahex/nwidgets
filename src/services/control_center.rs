use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCenterSection {
    Volume,
    Mic,
    Bluetooth,
    Network,
}

#[derive(Clone)]
pub struct ControlCenterStateChanged;

#[derive(Clone)]
pub struct ControlCenterService {
    visible: Arc<RwLock<bool>>,
    expanded_section: Arc<RwLock<Option<ControlCenterSection>>>,
    window_handle: Arc<RwLock<Option<AnyWindowHandle>>>,
}

impl EventEmitter<ControlCenterStateChanged> for ControlCenterService {}

impl ControlCenterService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            visible: Arc::new(RwLock::new(false)),
            expanded_section: Arc::new(RwLock::new(None)),
            window_handle: Arc::new(RwLock::new(None)),
        }
    }

    pub fn toggle(&self, cx: &mut Context<Self>) {
        let mut visible = self.visible.write();
        if *visible {
            *visible = false;
            self.close_window(cx);
        } else {
            *visible = true;
            let service = self.clone();
            cx.spawn(move |_, cx: &mut AsyncApp| {
                let cx = cx.clone();
                async move {
                    let _ = cx.update(|cx| {
                        service.open_window(cx);
                    });
                }
            })
            .detach();
        }

        cx.emit(ControlCenterStateChanged);
        cx.notify();
    }

    pub fn toggle_section(&self, section: ControlCenterSection, cx: &mut Context<Self>) {
        let mut current = self.expanded_section.write();
        if *current == Some(section) {
            *current = None;
        } else {
            *current = Some(section);
        }

        cx.emit(ControlCenterStateChanged);
        cx.notify();
    }

    pub fn expanded_section(&self) -> Option<ControlCenterSection> {
        *self.expanded_section.read()
    }

    fn open_window(&self, cx: &mut App) {
        if self.window_handle.read().is_some() {
            return;
        }

        let handle = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point {
                        x: px(0.0),
                        y: px(0.0),
                    },
                    size: Size {
                        width: px(600.0),
                        height: px(1370.0),
                    },
                })),
                titlebar: None,
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nwidgets-control-center".to_string(),
                    layer: Layer::Top,
                    anchor: Anchor::TOP | Anchor::RIGHT | Anchor::BOTTOM,
                    exclusive_zone: None,
                    margin: Some((px(40.0), px(10.0), px(20.0), px(0.0))),
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_window, cx| {
                use crate::widgets::control_center::ControlCenterWidget;
                cx.new(ControlCenterWidget::new)
            },
        );

        if let Ok(handle) = handle {
            let _ = handle.update(cx, |view, window, cx| {
                window.focus(&view.focus_handle, cx);
                cx.activate(true);
            });
            *self.window_handle.write() = Some(handle.into());
        }
    }

    fn close_window(&self, cx: &mut Context<Self>) {
        if let Some(handle) = self.window_handle.write().take() {
            let _ = handle.update(cx, |_, window, _| {
                window.remove_window();
            });
        }
    }
}

struct GlobalControlCenterService(Entity<ControlCenterService>);
impl Global for GlobalControlCenterService {}

impl ControlCenterService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalControlCenterService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalControlCenterService(service.clone()));
        service
    }
}
